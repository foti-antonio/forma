use crate::{agent_provider as engine, agent_types::*, domain::*, git_engine, service::Service};
use rusqlite::params;
use std::path::Path;

// Bounded, no symlink traversal. Unknown size is explicit rather than a partial total.
fn disk_usage(root: &Path) -> Option<u64> {
    let mut pending = vec![root.to_path_buf()];
    let mut bytes = 0u64;
    let mut count = 0;
    while let Some(path) = pending.pop() {
        count += 1;
        if count > 10000 {
            return None;
        }
        let meta = path.symlink_metadata().ok()?;
        if meta.is_symlink() {
            return None;
        }
        if meta.is_dir() {
            for entry in std::fs::read_dir(path).ok()? {
                pending.push(entry.ok()?.path());
                if pending.len() > 10000 {
                    return None;
                }
            }
        } else {
            bytes = bytes.checked_add(meta.len())?;
        }
    }
    Some(bytes)
}
fn dirty(repo: &git2::Repository) -> Result<bool> {
    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(true)
        .update_index(false);
    Ok(!repo
        .statuses(Some(&mut opts))
        .map_err(git_engine::failure)?
        .is_empty())
}
impl Service {
    pub fn agent_worktrees(&self, offset: usize) -> Result<Vec<WorktreeInfo>> {
        self.agent_recent(offset)?
            .into_iter()
            .map(|run| {
                let repository = self.repo.conn.query_row(
                    "SELECT path FROM git_repositories WHERE id=?1",
                    [&run.repository_id],
                    |r| r.get(0),
                )?;
                let (cleanup_state, message): (String, String) = self.repo.conn.query_row(
                    "SELECT cleanup_state,cleanup_message FROM agent_runs WHERE id=?1",
                    [&run.id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?;
                let checked = self.agent_worktree(&run);
                let (dirty, disk_bytes, message) = match checked {
                    Ok(repo) => (
                        dirty(&repo).ok(),
                        disk_usage(Path::new(&run.worktree)),
                        message,
                    ),
                    Err(e) => (
                        None,
                        None,
                        if message.is_empty() {
                            e.message
                        } else {
                            message
                        },
                    ),
                };
                Ok(WorktreeInfo {
                    run,
                    repository,
                    cleanup_state,
                    message,
                    dirty,
                    disk_bytes,
                })
            })
            .collect()
    }
    pub fn agent_cleanup(&self, run: &str, confirmed: bool) -> Result<()> {
        id(run)?;
        if !confirmed {
            return Err(engine::error("Confirm worktree deletion explicitly."));
        }
        let record = self.repo.agent_run(run)?;
        if matches!(record.status.as_str(), "Queued" | "Running") || record.review.is_none() {
            return Err(engine::error("Stop and review the run before cleanup."));
        }
        if self
            .repo
            .conn
            .prepare("SELECT 1 FROM independent_tests WHERE run_id=?1 AND status='Running'")?
            .exists([run])?
        {
            return Err(engine::error("Stop independent tests before cleanup."));
        }
        let state: String = self.repo.conn.query_row(
            "SELECT cleanup_state FROM agent_runs WHERE id=?1",
            [run],
            |r| r.get(0),
        )?;
        if state == "Removed" {
            return Ok(());
        }
        // Recover a partially completed cleanup without force-removing a path.
        if matches!(state.as_str(), "Removing" | "Failed")
            && Path::new(&record.worktree)
                .symlink_metadata()
                .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        {
            let expected = self.agent_root()?.join(run);
            if !record.trusted || expected != Path::new(&record.worktree) {
                return Err(engine::error("Cleanup path is not authorized."));
            }
            let project: String = self.repo.conn.query_row(
                "SELECT project_id FROM issues WHERE id=?1",
                [&record.issue_id],
                |r| r.get(0),
            )?;
            let source_record = self.repo.git_repository(&project, &record.repository_id)?;
            if !source_record.trusted {
                return Err(engine::error(
                    "Reauthorize the repository before cleanup recovery.",
                ));
            }
            let source = git_engine::open(Path::new(&source_record.path))?;
            match source.find_worktree(&format!("forma-{run}")) {
                Ok(linked) => {
                    if linked.path() != expected {
                        return Err(engine::error("Registered worktree path changed."));
                    }
                    linked.prune(None).map_err(git_engine::failure)?;
                }
                Err(e) if e.code() == git2::ErrorCode::NotFound => {}
                Err(e) => return Err(git_engine::failure(e)),
            }
            self.repo.conn.execute("UPDATE agent_runs SET cleanup_state='Removed',cleanup_message='Interrupted cleanup recovered; branch and history retained.' WHERE id=?1",[run])?;
            return Ok(());
        }
        let work = self.agent_worktree(&record)?;
        if !work.is_worktree() || dirty(&work)? || disk_usage(Path::new(&record.worktree)).is_none()
        {
            return Err(engine::error("Cleanup requires a known clean linked worktree without symlinks; preserve modified, ignored or unknown files manually first."));
        }
        let project: String = self.repo.conn.query_row(
            "SELECT project_id FROM issues WHERE id=?1",
            [&record.issue_id],
            |r| r.get(0),
        )?;
        let source_record = self.repo.git_repository(&project, &record.repository_id)?;
        let source = git_engine::open(Path::new(&source_record.path))?;
        let linked = source
            .find_worktree(&format!("forma-{run}"))
            .map_err(git_engine::failure)?;
        if linked.path().canonicalize().ok() != Path::new(&record.worktree).canonicalize().ok()
            || !matches!(
                linked.is_locked().map_err(git_engine::failure)?,
                git2::WorktreeLockStatus::Unlocked
            )
        {
            return Err(engine::error(
                "Worktree identity changed or is locked. Inspect manually.",
            ));
        }
        self.repo.conn.execute(
            "UPDATE agent_runs SET cleanup_state='Removing',cleanup_message='' WHERE id=?1",
            [run],
        )?;
        // Only a validated, reviewed, clean registered worktree reaches libgit2.
        // Branch and historical run metadata remain; no force-remove endpoint exists.
        let mut options = git2::WorktreePruneOptions::new();
        options.valid(true).working_tree(true);
        drop(work);
        let result = linked
            .prune(Some(&mut options))
            .map_err(git_engine::failure);
        let (state, message) = if result.is_ok() {
            ("Removed", "Worktree removed; branch and history retained.")
        } else {
            (
                "Failed",
                "Cleanup failed. Retained metadata allows retry; inspect the path before retrying.",
            )
        };
        self.repo.conn.execute(
            "UPDATE agent_runs SET cleanup_state=?2,cleanup_message=?3 WHERE id=?1",
            params![run, state, message],
        )?;
        result
    }
}
