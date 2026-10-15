//! Local read-only libgit2 operations. No remote, subprocess, hook or mutation API.
use crate::domain::*;
use git2::{BranchType, ErrorCode, Repository, RepositoryOpenFlags, Status, StatusOptions};
use std::path::Path;

pub(crate) fn failure(_: git2::Error) -> AppError {
    // Git errors can contain credentials, config values or private paths.
    AppError { code: "git", message: "Could not read this repository. It may be missing, inaccessible or damaged. Check it and refresh, or reconnect a folder.".into() }
}
pub(crate) fn open(path: &Path) -> Result<Repository> {
    Repository::open_ext(
        path,
        RepositoryOpenFlags::NO_SEARCH,
        std::iter::empty::<&Path>(),
    )
    .map_err(failure)
}
fn text(bytes: &[u8], limit: usize) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .take(limit)
        .collect()
}
pub(crate) fn commit(c: &git2::Commit<'_>) -> GitCommit {
    GitCommit {
        hash: c.id().to_string(),
        message: text(c.message_bytes(), 4000),
        author: text(c.author().name_bytes(), 200),
        committed_at: chrono::DateTime::from_timestamp(c.time().seconds(), 0)
            .map(|t| t.to_rfc3339())
            .unwrap_or_else(|| "Unknown timestamp".into()),
    }
}
pub(crate) fn target(repo: &Repository, kind: &str, value: &str) -> Result<GitCommit> {
    let c = if kind == "branch" {
        repo.find_branch(value, BranchType::Local)
            .map_err(failure)?
            .get()
            .peel_to_commit()
            .map_err(failure)?
    } else {
        let oid = git2::Oid::from_str(value).map_err(failure)?;
        repo.find_commit(oid).map_err(failure)?
    };
    Ok(commit(&c))
}
pub(crate) fn overview(repo: &Repository, offset: usize, limit: usize) -> Result<GitOverview> {
    let head = match repo.head() {
        Ok(h) => Some(h),
        Err(e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => None,
        Err(e) => return Err(failure(e)),
    };
    let branch = if let Some(h) = &head {
        if h.is_branch() {
            text(h.shorthand_bytes(), 1024)
        } else {
            format!(
                "Detached HEAD · {}",
                h.target()
                    .map(|id| id.to_string()[..12].to_owned())
                    .unwrap_or_default()
            )
        }
    } else {
        let name = repo
            .find_reference("HEAD")
            .ok()
            .and_then(|h| h.symbolic_target().ok().flatten().map(str::to_owned));
        format!(
            "{} (unborn)",
            name.unwrap_or_else(|| "HEAD".into())
                .trim_start_matches("refs/heads/")
        )
    };
    let status = if repo.is_bare() {
        None
    } else {
        let mut options = StatusOptions::new();
        options
            .include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false)
            .exclude_submodules(true)
            .renames_head_to_index(false)
            .renames_index_to_workdir(false)
            .update_index(false);
        let statuses = repo.statuses(Some(&mut options)).map_err(failure)?;
        let mut counts = GitStatus::default();
        for entry in statuses.iter() {
            let s = entry.status();
            counts.untracked += usize::from(s.contains(Status::WT_NEW));
            counts.added += usize::from(s.contains(Status::INDEX_NEW));
            counts.deleted += usize::from(s.intersects(Status::WT_DELETED | Status::INDEX_DELETED));
            counts.modified += usize::from(s.intersects(
                Status::WT_MODIFIED
                    | Status::INDEX_MODIFIED
                    | Status::WT_TYPECHANGE
                    | Status::INDEX_TYPECHANGE
                    | Status::INDEX_RENAMED
                    | Status::WT_RENAMED,
            ));
            counts.conflicted += usize::from(s.contains(Status::CONFLICTED));
        }
        Some(counts)
    };
    let mut commits = Vec::new();
    if let Some(h) = head {
        let mut walk = repo.revwalk().map_err(failure)?;
        // Topological order avoids a full-history time sort before returning a page.
        walk.push(h.peel_to_commit().map_err(failure)?.id())
            .map_err(failure)?;
        for oid in walk.skip(offset).take(limit + 1) {
            commits.push(commit(
                &repo.find_commit(oid.map_err(failure)?).map_err(failure)?,
            ));
        }
    }
    let has_more = commits.len() > limit;
    commits.truncate(limit);
    Ok(GitOverview {
        branch,
        bare: repo.is_bare(),
        status,
        commits,
        has_more,
    })
}
pub(crate) fn branches(
    repo: &Repository,
    search: &str,
    offset: usize,
    limit: usize,
) -> Result<GitBranchPage> {
    let mut branches = Vec::new();
    let mut matched = 0;
    let mut truncated = false;
    for (scanned, branch) in repo
        .branches(Some(BranchType::Local))
        .map_err(failure)?
        .enumerate()
    {
        if scanned == 10000 {
            truncated = true;
            break;
        }
        let (b, _) = branch.map_err(failure)?;
        let name = b
            .name()
            .map_err(failure)?
            .ok_or_else(|| AppError::validation("A branch name is not valid UTF-8."))?;
        if name.len() > 1024 {
            return Err(AppError::validation(
                "A branch name exceeds the supported length.",
            ));
        }
        if !name.to_lowercase().contains(&search.to_lowercase()) {
            continue;
        }
        if matched >= offset {
            branches.push(GitBranch {
                name: name.into(),
                active: b.is_head(),
                commit: commit(&b.get().peel_to_commit().map_err(failure)?),
            });
            if branches.len() > limit {
                break;
            }
        }
        matched += 1;
    }
    let has_more = branches.len() > limit;
    branches.truncate(limit);
    Ok(GitBranchPage {
        branches,
        has_more,
        truncated,
    })
}
