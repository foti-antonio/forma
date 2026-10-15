use crate::{
    agent_tests::{fixture_executable, setup, wait},
    agent_types::*,
    domain::*,
    service::Service,
};
use std::{
    path::Path,
    time::{Duration, Instant},
};
fn another(s: &mut Service, p: &str, title: &str) -> Issue {
    s.save_issue(
        None,
        IssueInput {
            project_id: p.into(),
            title: title.into(),
            description: "Acceptance criteria".into(),
            status: "todo".into(),
            priority: "none".into(),
            due_date: None,
            parent_id: None,
            label_ids: vec![],
            position: 0,
        },
    )
    .unwrap()
}
fn enqueue(s: &mut Service, p: &str, i: &Issue) -> String {
    let preview = s.agent_preview(p, &i.id, "codex").unwrap();
    s.agent_enqueue(&preview.token, true, false).unwrap()
}
fn test_wait(s: &Service, run: &str) -> IndependentTest {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let mut tests = s.test_runs(run).unwrap();
        let test = tests.remove(0);
        if test.status != "Running" {
            return test;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(20));
    }
}
#[test]
fn two_approved_same_repository_runs_cancel_independently_and_preserve_primary() {
    let (dir, mut s, p, i) = setup("SLOW_EXECUTION SPAWN_DESCENDANT");
    assert_eq!(s.agent_concurrency().unwrap(), 1);
    assert!(s.agent_set_concurrency(3).is_err());
    let j = another(&mut s, &p, "SLOW_EXECUTION");
    enqueue(&mut s, &p, &i);
    enqueue(&mut s, &p, &j);
    assert!(s.agent_recent(0).unwrap().is_empty()); // Approval never launches.
    s.agent_set_concurrency(2).unwrap();
    let runs = s.agent_start_queue(true).unwrap();
    assert_eq!(runs.len(), 2);
    assert_ne!(runs[0].branch, runs[1].branch);
    assert_ne!(runs[0].worktree, runs[1].worktree);
    let third = another(&mut s, &p, "third");
    let preview = s.agent_preview(&p, &third.id, "codex").unwrap();
    assert!(s.agent_start(&preview.token, true, false).is_err());
    let duplicate = s.agent_preview(&p, &i.id, "codex").unwrap();
    assert!(s.agent_start(&duplicate.token, true, false).is_err());
    s.agent_cancel(&runs[0].id).unwrap();
    assert_eq!(wait(&s, &runs[0].id).status, "Cancelled");
    assert!(matches!(
        s.repo.agent_run(&runs[1].id).unwrap().status.as_str(),
        "Queued" | "Running"
    ));
    s.agent_cancel(&runs[1].id).unwrap();
    assert_eq!(wait(&s, &runs[1].id).status, "Cancelled");
    assert!(Path::new(&runs[0].worktree).is_dir());
    assert!(!dir.path().join("repo/received-args.txt").exists());
    assert_eq!(s.issue(&p, &i.id).unwrap().status, "todo");
}
#[test]
fn different_repositories_execute_concurrently() {
    let (_dir, mut s, p, i) = setup("SLOW_EXECUTION");
    let (_other, mut other, op, oi) = setup("Other repository");
    // Same application, second project attached to the other real repository.
    let workspace = s.snapshot().unwrap().active_workspace_id.unwrap();
    let project = s
        .save_project(
            None,
            ProjectInput {
                workspace_id: workspace,
                name: "Second".into(),
                description: String::new(),
                icon: "code".into(),
                color: "#112233".into(),
                status: "active".into(),
            },
        )
        .unwrap()
        .projects
        .into_iter()
        .find(|p| p.name == "Second")
        .unwrap();
    let other_path = other.git_connection(&op).unwrap().unwrap().path;
    s.git_connect(&project.id, Path::new(&other_path)).unwrap();
    let j = another(&mut s, &project.id, &oi.title);
    s.agent_set_concurrency(2).unwrap();
    enqueue(&mut s, &p, &i);
    enqueue(&mut s, &project.id, &j);
    let runs = s.agent_start_queue(true).unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(wait(&s, &runs[1].id).status, "Completed");
    s.agent_cancel(&runs[0].id).unwrap();
    assert_eq!(wait(&s, &runs[0].id).status, "Cancelled");
    // Keep second Service mutable use out of this application's execution lifecycle.
    other.agent_queue_stop();
}
#[test]
fn queue_reorder_cancel_expire_invalidate_context_and_restart() {
    let (dir, mut s, p, i) = setup("first");
    let j = another(&mut s, &p, "second");
    let a = enqueue(&mut s, &p, &i);
    let b = enqueue(&mut s, &p, &j);
    let duplicate = s.agent_preview(&p, &i.id, "codex").unwrap();
    assert!(s.agent_enqueue(&duplicate.token, true, false).is_err());
    s.agent_queue_move(&b, Some(&a)).unwrap();
    assert_eq!(s.agent_queue().unwrap()[0].id, b);
    s.agent_queue_cancel(&b).unwrap();
    s.repo
        .conn
        .execute(
            "UPDATE projects SET description='Changed context' WHERE id=?1",
            [&p],
        )
        .unwrap();
    assert!(s.agent_start_queue(true).unwrap().is_empty());
    assert_eq!(
        s.agent_queue()
            .unwrap()
            .iter()
            .find(|t| t.id == a)
            .unwrap()
            .status,
        "Invalidated"
    );
    let c = enqueue(&mut s, &p, &i);
    s.repo
        .conn
        .execute(
            "UPDATE agent_queue SET expires_at='2000-01-01T00:00:00Z' WHERE id=?1",
            [&c],
        )
        .unwrap();
    assert!(s.agent_start_queue(true).unwrap().is_empty());
    let d = enqueue(&mut s, &p, &i);
    drop(s);
    let s = Service::open(&dir.path().join("test.sqlite3")).unwrap();
    assert!(s.agent_recent(0).unwrap().is_empty());
    assert_eq!(
        s.agent_queue()
            .unwrap()
            .iter()
            .find(|t| t.id == d)
            .unwrap()
            .status,
        "Invalidated"
    );
}
#[test]
fn explicit_dispatch_drains_only_original_approved_set_and_can_stop() {
    let (_dir, mut s, p, i) = setup("one");
    let j = another(&mut s, &p, "two");
    let k = another(&mut s, &p, "three");
    enqueue(&mut s, &p, &i);
    enqueue(&mut s, &p, &j);
    let runs = s.agent_queue_begin().unwrap();
    assert_eq!(runs.len(), 1);
    let late = enqueue(&mut s, &p, &k);
    wait(&s, &runs[0].id);
    assert!(!s.agent_queue_tick().unwrap());
    let next = s
        .agent_recent(0)
        .unwrap()
        .into_iter()
        .find(|r| r.issue_id == j.id)
        .unwrap();
    wait(&s, &next.id);
    assert_eq!(
        s.agent_queue()
            .unwrap()
            .iter()
            .find(|t| t.id == late)
            .unwrap()
            .status,
        "Approved"
    );
    assert!(s.agent_queue_begin().is_ok());
    s.agent_queue_stop();
    assert!(!s.agent_queue_tick().unwrap());
    for run in s.agent_recent(0).unwrap() {
        wait(&s, &run.id);
    }
}
#[test]
fn independently_approved_tests_capture_failure_cancel_limits_and_restore_trust() {
    let (dir, mut s, p, i) = setup("Task");
    let preview = s.agent_preview(&p, &i.id, "codex").unwrap();
    let run = s.agent_start(&preview.token, true, false).unwrap();
    wait(&s, &run.id);
    s.test_authorize(&p, &fixture_executable()).unwrap();
    let mut cmd = s.test_command(&p).unwrap().unwrap();
    cmd.arguments = vec![
        "--forma-test".into(),
        "fail".into(),
        "$(touch injected); echo nope".into(),
    ];
    s.test_save_command(cmd.clone()).unwrap();
    let preview = s.test_preview(&run.id).unwrap();
    assert_eq!(preview.working_directory, run.worktree);
    assert!(s.test_start(&preview.token, false).is_err());
    s.test_start(&preview.token, true).unwrap();
    assert!(s.test_start(&preview.token, true).is_err());
    let done = test_wait(&s, &run.id);
    assert_eq!(done.status, "Failed");
    assert_eq!(done.exit_code, Some(9));
    assert!(done.duration_ms.is_some());
    assert!(done.log.contains("independent test output"));
    assert!(!Path::new(&run.worktree).join("injected").exists());
    cmd.arguments = vec!["--forma-test".into(), "slow".into()];
    s.test_save_command(cmd.clone()).unwrap();
    let preview = s.test_preview(&run.id).unwrap();
    let tid = s.test_start(&preview.token, true).unwrap();
    assert!(s.delete_issue(&p, &i.id).is_err());
    let j = another(&mut s, &p, "another");
    let preview = s.agent_preview(&p, &j.id, "codex").unwrap();
    assert!(s.agent_start(&preview.token, true, false).is_err());
    s.test_cancel(&tid).unwrap();
    assert_eq!(test_wait(&s, &run.id).status, "Cancelled");
    cmd.arguments = vec!["--forma-test".into()];
    s.test_save_command(cmd).unwrap();
    let preview = s.test_preview(&run.id).unwrap();
    s.test_start(&preview.token, true).unwrap();
    assert_eq!(test_wait(&s, &run.id).status, "Completed");
    assert_eq!(s.issue(&p, &i.id).unwrap().status, "todo");
    let backup = dir.path().join("m06.forma-backup");
    s.export_backup(&backup).unwrap();
    s.prepare_restore(&backup).unwrap();
    s.restore_backup(true).unwrap();
    assert!(!s.test_command(&p).unwrap().unwrap().trusted);
    assert!(s.test_preview(&run.id).is_err());
    assert!(!s.test_runs(&run.id).unwrap().is_empty());
}
#[test]
fn clean_reviewed_worktree_cleanup_preserves_branch_and_history() {
    let (_dir, mut s, p, i) = setup("Task");
    let preview = s.agent_preview(&p, &i.id, "codex").unwrap();
    let run = s.agent_start(&preview.token, true, false).unwrap();
    wait(&s, &run.id);
    assert!(s.agent_cleanup(&run.id, true).is_err());
    s.agent_mark_reviewed(&run.id, "accepted").unwrap();
    assert!(s.agent_cleanup(&run.id, true).is_err());
    for name in ["agent-result.txt", "received-args.txt", "process-id.txt"] {
        std::fs::remove_file(Path::new(&run.worktree).join(name)).unwrap();
    }
    assert!(s.agent_cleanup(&run.id, false).is_err());
    s.agent_cleanup(&run.id, true).unwrap();
    assert!(!Path::new(&run.worktree).exists());
    assert_eq!(
        s.repo.agent_run(&run.id).unwrap().review.as_deref(),
        Some("accepted")
    );
    let source = git2::Repository::open(s.git_connection(&p).unwrap().unwrap().path).unwrap();
    assert!(source
        .find_branch(&run.branch, git2::BranchType::Local)
        .is_ok());
    s.agent_cleanup(&run.id, true).unwrap();
    assert_eq!(s.agent_worktrees(0).unwrap()[0].cleanup_state, "Removed");
}
#[test]
fn whole_process_os_isolation_requirement_fails_closed() {
    let (_dir, mut s, p, i) = setup("Task");
    let mut setting = s
        .agent_settings()
        .unwrap()
        .into_iter()
        .find(|x| x.provider == "codex")
        .unwrap();
    setting.require_os_isolation = true;
    s.agent_save_settings(setting).unwrap();
    let error = s.agent_preview(&p, &i.id, "codex").unwrap_err();
    assert!(error.message.contains("unavailable"));
    assert!(s.agent_recent(0).unwrap().is_empty());
}

#[test]
fn noisy_run_hits_output_limit_without_starving_other_run() {
    let (_dir, mut s, p, i) = setup("FLOOD_OUTPUT LONG_LINE");
    let j = another(&mut s, &p, "SLOW_EXECUTION");
    let mut setting = s
        .agent_settings()
        .unwrap()
        .into_iter()
        .find(|s| s.provider == "codex")
        .unwrap();
    setting.log_limit = 1024;
    s.agent_save_settings(setting).unwrap();
    s.agent_set_concurrency(2).unwrap();
    enqueue(&mut s, &p, &i);
    enqueue(&mut s, &p, &j);
    let start = Instant::now();
    let runs = s.agent_start_queue(true).unwrap();
    let noisy = wait(&s, &runs[0].id);
    assert_eq!(noisy.status, "Failed");
    assert!(noisy.result.contains("Output limit"));
    assert!(noisy.log.len() <= 1024);
    assert!(start.elapsed() < Duration::from_secs(5));
    assert!(matches!(
        s.repo.agent_run(&runs[1].id).unwrap().status.as_str(),
        "Running" | "Queued"
    ));
    s.agent_cancel(&runs[1].id).unwrap();
    assert_eq!(wait(&s, &runs[1].id).status, "Cancelled");
}
#[test]
fn cleanup_rejects_locked_missing_corrupt_and_primary_worktrees_and_retries() {
    let (dir, mut s, p, i) = setup("Task");
    let preview = s.agent_preview(&p, &i.id, "codex").unwrap();
    let run = s.agent_start(&preview.token, true, false).unwrap();
    wait(&s, &run.id);
    s.agent_mark_reviewed(&run.id, "accepted").unwrap();
    for name in ["agent-result.txt", "received-args.txt", "process-id.txt"] {
        std::fs::remove_file(Path::new(&run.worktree).join(name)).unwrap();
    }
    let source = git2::Repository::open(dir.path().join("repo")).unwrap();
    let linked = source.find_worktree(&format!("forma-{}", run.id)).unwrap();
    linked.lock(Some("Retain for inspection")).unwrap();
    assert!(s.agent_cleanup(&run.id, true).is_err());
    linked.unlock().unwrap();
    let git_file = Path::new(&run.worktree).join(".git");
    let original = std::fs::read(&git_file).unwrap();
    std::fs::write(&git_file, "corrupt").unwrap();
    assert!(s.agent_cleanup(&run.id, true).is_err());
    assert!(s.agent_worktrees(0).unwrap()[0].dirty.is_none());
    std::fs::write(&git_file, original).unwrap();
    s.repo
        .conn
        .execute(
            "UPDATE agent_runs SET worktree=?2 WHERE id=?1",
            rusqlite::params![run.id, dir.path().join("repo").to_string_lossy()],
        )
        .unwrap();
    assert!(s.agent_cleanup(&run.id, true).is_err());
    assert!(dir.path().join("repo/README.md").is_file());
    s.repo
        .conn
        .execute(
            "UPDATE agent_runs SET worktree=?2 WHERE id=?1",
            rusqlite::params![run.id, run.worktree],
        )
        .unwrap();
    let renamed = dir.path().join("retained");
    std::fs::rename(&run.worktree, &renamed).unwrap();
    assert!(s.agent_cleanup(&run.id, true).is_err());
    std::fs::rename(renamed, &run.worktree).unwrap();
    s.agent_cleanup(&run.id, true).unwrap();
}
#[test]
fn interrupted_tests_and_restored_queue_never_resume_or_restore_authorizations() {
    let (dir, mut s, p, i) = setup("Task");
    let queued = enqueue(&mut s, &p, &i);
    let backup = dir.path().join("queue.forma-backup");
    s.export_backup(&backup).unwrap();
    s.agent_start_queue(false).unwrap();
    let run = s.agent_recent(0).unwrap().remove(0);
    wait(&s, &run.id);
    s.test_authorize(&p, &fixture_executable()).unwrap();
    let mut cmd = s.test_command(&p).unwrap().unwrap();
    cmd.arguments = vec!["--forma-test".into()];
    s.test_save_command(cmd).unwrap();
    let preview = s.test_preview(&run.id).unwrap();
    let tid = s.test_start(&preview.token, true).unwrap();
    test_wait(&s, &run.id);
    s.repo
        .conn
        .execute(
            "UPDATE independent_tests SET status='Running',ended_at=NULL WHERE id=?1",
            [tid],
        )
        .unwrap();
    drop(s);
    let mut s = Service::open(&dir.path().join("test.sqlite3")).unwrap();
    assert_eq!(s.test_runs(&run.id).unwrap()[0].status, "Interrupted");
    s.prepare_restore(&backup).unwrap();
    s.restore_backup(true).unwrap();
    assert_eq!(
        s.agent_queue()
            .unwrap()
            .into_iter()
            .find(|t| t.id == queued)
            .unwrap()
            .status,
        "Invalidated"
    );
    assert!(s.agent_start_queue(true).unwrap().is_empty());
    assert!(s.agent_recent(0).unwrap().is_empty());
}
#[test]
fn migration_006_preserves_m05_and_rolls_back_failed_upgrade() {
    let (dir, mut s, p, i) = setup("M05 data");
    let backup = dir.path().join("m05.forma-backup");
    s.export_backup(&backup).unwrap();
    let c = rusqlite::Connection::open(&backup).unwrap();
    c.execute_batch("DROP TABLE independent_tests; DROP TABLE project_test_commands; DROP TABLE agent_queue; ALTER TABLE agent_runs DROP COLUMN cleanup_state; ALTER TABLE agent_runs DROP COLUMN cleanup_message; ALTER TABLE agent_settings DROP COLUMN require_os_isolation; ALTER TABLE agent_preferences DROP COLUMN max_concurrent; PRAGMA user_version=5; UPDATE backup_metadata SET format_version=3;").unwrap();
    drop(c);
    s.prepare_restore(&backup).unwrap();
    s.restore_backup(true).unwrap();
    assert_eq!(s.issue(&p, &i.id).unwrap().title, "M05 data");
    assert_eq!(s.agent_concurrency().unwrap(), 1);
    let broken = dir.path().join("broken006.sqlite3");
    std::fs::copy(&backup, &broken).unwrap();
    let c = rusqlite::Connection::open(&broken).unwrap();
    c.execute_batch("CREATE TABLE agent_queue(collision)")
        .unwrap();
    drop(c);
    assert!(Service::open(&broken).is_err());
    let c = rusqlite::Connection::open(&broken).unwrap();
    assert_eq!(
        c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        5
    );
    assert!(c
        .prepare("SELECT max_concurrent FROM agent_preferences")
        .is_err());
    c.execute_batch("DROP TABLE agent_queue").unwrap();
    drop(c);
    assert_eq!(
        Service::open(&broken)
            .unwrap()
            .issue(&p, &i.id)
            .unwrap()
            .title,
        "M05 data"
    );
}
#[cfg(target_os = "linux")]
#[test]
#[ignore = "manual concurrent process CPU/RSS sampling"]
fn concurrent_process_measurements() {
    let (_dir, mut s, p, i) = setup("CPU_EXECUTION");
    let j = another(&mut s, &p, "CPU_EXECUTION");
    s.agent_set_concurrency(2).unwrap();
    enqueue(&mut s, &p, &i);
    enqueue(&mut s, &p, &j);
    let start = Instant::now();
    let runs = s.agent_start_queue(true).unwrap();
    let mut peaks = [0u64; 2];
    let mut ticks = [0u64; 2];
    let mut peak_sum = 0;
    while start.elapsed() < Duration::from_secs(5) {
        let mut sum = 0;
        for (index, run) in runs.iter().enumerate() {
            if let Ok(pid) =
                std::fs::read_to_string(Path::new(&run.worktree).join("process-id.txt"))
            {
                if let Ok(status) = std::fs::read_to_string(format!("/proc/{pid}/status")) {
                    let rss = status
                        .lines()
                        .find_map(|l| l.strip_prefix("VmRSS:"))
                        .and_then(|l| l.split_whitespace().next())
                        .and_then(|n| n.parse::<u64>().ok())
                        .unwrap_or(0);
                    peaks[index] = peaks[index].max(rss);
                    sum += rss;
                }
                if let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) {
                    if let Some((_, fields)) = stat.rsplit_once(") ") {
                        let fields: Vec<_> = fields.split_whitespace().collect();
                        ticks[index] = ticks[index].max(
                            fields[11].parse::<u64>().unwrap() + fields[12].parse::<u64>().unwrap(),
                        );
                    }
                }
            }
        }
        peak_sum = peak_sum.max(sum);
        if runs
            .iter()
            .all(|r| s.repo.agent_run(&r.id).unwrap().status == "Completed")
        {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    for run in &runs {
        assert_eq!(wait(&s, &run.id).status, "Completed");
    }
    println!("two CPU fixture runs: wall={:.3}s, leader peak RSS KiB={peaks:?}, simultaneous peak sum={peak_sum} KiB, sampled CPU ticks={ticks:?}, clock ticks/sec={hz}. Excludes Forma, watchdogs and descendants; sampling is a lower bound, not a resource quota.",start.elapsed().as_secs_f64());
}

#[test]
fn interrupted_cleanup_recovers_registration_without_force_removal() {
    let (dir, mut s, p, i) = setup("Task");
    let preview = s.agent_preview(&p, &i.id, "codex").unwrap();
    let run = s.agent_start(&preview.token, true, false).unwrap();
    wait(&s, &run.id);
    s.agent_mark_reviewed(&run.id, "accepted").unwrap();
    s.repo
        .conn
        .execute(
            "UPDATE agent_runs SET cleanup_state='Removing' WHERE id=?1",
            [&run.id],
        )
        .unwrap();
    // Simulate the point after removing the known temporary fixture folder but
    // before deleting libgit2 registration/updating the database.
    std::fs::remove_dir_all(&run.worktree).unwrap();
    drop(s);
    let s = Service::open(&dir.path().join("test.sqlite3")).unwrap();
    assert_eq!(s.agent_worktrees(0).unwrap()[0].cleanup_state, "Failed");
    s.agent_cleanup(&run.id, true).unwrap();
    assert_eq!(s.agent_worktrees(0).unwrap()[0].cleanup_state, "Removed");
}
#[cfg(unix)]
#[test]
fn replaced_executable_symlink_invalidates_queued_approval() {
    use std::os::unix::fs::symlink;
    let (dir, mut s, p, i) = setup("Task");
    let selected = dir.path().join("selected-codex");
    std::fs::copy(fixture_executable(), &selected).unwrap();
    s.agent_authorize("codex", &selected).unwrap();
    enqueue(&mut s, &p, &i);
    std::fs::remove_file(&selected).unwrap();
    symlink(fixture_executable(), &selected).unwrap();
    assert!(s.agent_start_queue(true).unwrap().is_empty());
    assert_eq!(s.agent_queue().unwrap()[0].status, "Invalidated");
    assert!(s.agent_recent(0).unwrap().is_empty());
}

#[test]
fn cleanup_and_creation_while_another_run_uses_the_same_repository() {
    let (_dir, mut s, p, i) = setup("SLOW_EXECUTION");
    let j = another(&mut s, &p, "Reviewable task");
    s.agent_set_concurrency(2).unwrap();
    enqueue(&mut s, &p, &i);
    enqueue(&mut s, &p, &j);
    let runs = s.agent_start_queue(true).unwrap();
    assert_eq!(wait(&s, &runs[1].id).status, "Completed");
    s.agent_mark_reviewed(&runs[1].id, "accepted").unwrap();
    for name in ["agent-result.txt", "received-args.txt", "process-id.txt"] {
        std::fs::remove_file(Path::new(&runs[1].worktree).join(name)).unwrap();
    }
    s.agent_cleanup(&runs[1].id, true).unwrap();
    assert!(matches!(
        s.repo.agent_run(&runs[0].id).unwrap().status.as_str(),
        "Queued" | "Running"
    ));
    let k = another(&mut s, &p, "Next task");
    let preview = s.agent_preview(&p, &k.id, "codex").unwrap();
    let next = s.agent_start(&preview.token, true, false).unwrap();
    assert_eq!(wait(&s, &next.id).status, "Completed");
    s.agent_cancel(&runs[0].id).unwrap();
    assert_eq!(wait(&s, &runs[0].id).status, "Cancelled");
}
#[test]
fn cancelled_before_spawn_does_not_execute_selected_binary() {
    use std::sync::{atomic::AtomicBool, Arc};
    let dir = tempfile::tempdir().unwrap();
    let result = crate::agent_provider::execute(
        &fixture_executable(),
        dir.path(),
        &["--forma-test".into()],
        "",
        crate::agent_provider::Limits {
            runtime: 1,
            output_bytes: 1024,
        },
        Arc::new(AtomicBool::new(true)),
        |_| {},
    )
    .unwrap();
    assert_eq!(result.status, "Cancelled");
    assert!(!dir.path().join("independent-arguments.txt").exists());
}

#[test]
fn failed_dispatch_releases_session_for_explicit_retry() {
    let (_dir, mut service, project, issue) = setup("Retry dispatch");
    enqueue(&mut service, &project, &issue);
    service.repo.conn.execute_batch("CREATE TRIGGER fail_queue_launch BEFORE UPDATE OF status ON agent_queue WHEN NEW.status='Launching' BEGIN SELECT RAISE(ABORT,'simulated queue write failure'); END;").unwrap();
    assert!(service.agent_queue_begin().is_err());
    assert!(!service.agent_queue_tick().unwrap());
    assert_eq!(service.agent_queue().unwrap()[0].status, "Approved");
    assert!(service.agent_recent(0).unwrap().is_empty());
    service
        .repo
        .conn
        .execute_batch("DROP TRIGGER fail_queue_launch")
        .unwrap();
    let runs = service.agent_queue_begin().unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(wait(&service, &runs[0].id).status, "Completed");
    service.agent_queue_stop();
}
