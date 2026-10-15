use crate::{domain::*, service::Service};
use git2::{Repository, Signature};
use std::path::Path;

fn setup() -> (tempfile::TempDir, Service, String, Issue) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Service::open(&dir.path().join("forma.sqlite3")).unwrap();
    let w = s
        .create_workspace("Git tests")
        .unwrap()
        .active_workspace_id
        .unwrap();
    let p = s
        .save_project(
            None,
            ProjectInput {
                workspace_id: w,
                name: "Git project".into(),
                description: "".into(),
                icon: "code".into(),
                color: "#123456".into(),
                status: "active".into(),
            },
        )
        .unwrap()
        .projects[0]
        .id
        .clone();
    let issue = s
        .save_issue(
            None,
            IssueInput {
                project_id: p.clone(),
                title: "Local Git".into(),
                description: "".into(),
                status: "todo".into(),
                priority: "none".into(),
                due_date: None,
                parent_id: None,
                label_ids: vec![],
                position: 0,
            },
        )
        .unwrap();
    (dir, s, p, issue)
}
fn commit(repo: &Repository, message: &str) -> git2::Oid {
    let mut index = repo.index().unwrap();
    index
        .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let sig = Signature::new(
        "Local Author",
        "local@example.test",
        &git2::Time::new(1700000000, 120),
    )
    .unwrap();
    let parent = repo.head().ok().map(|h| h.peel_to_commit().unwrap());
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        message,
        &tree,
        &parent.iter().collect::<Vec<_>>(),
    )
    .unwrap()
}
fn fixture(path: &Path) -> (Repository, git2::Oid) {
    let repo = Repository::init(path).unwrap();
    std::fs::write(path.join("tracked"), "original\n").unwrap();
    std::fs::write(path.join("delete"), "delete me\n").unwrap();
    let oid = commit(&repo, "Initial F1-1\n\nFull message");
    (repo, oid)
}
fn input(p: &str, i: &Issue, r: &GitRepository, kind: &str, target: &str) -> GitLinkInput {
    GitLinkInput {
        project_id: p.into(),
        issue_id: i.id.clone(),
        repository_id: r.id.clone(),
        kind: kind.into(),
        target: target.into(),
        confirmed: true,
    }
}
#[test]
fn repository_validation_status_history_branches_and_no_writes() {
    let (dir, mut s, p, _) = setup();
    assert!(s.git_connect(&p, dir.path()).is_err());
    assert!(s.git_connect(&p, &dir.path().join("missing")).is_err());
    let root = dir.path().join("repo");
    let (repo, first) = fixture(&root);
    let branch = "feature/F1-1".to_string();
    repo.branch(&branch, &repo.find_commit(first).unwrap(), false)
        .unwrap();
    std::fs::write(root.join("tracked"), "changed").unwrap();
    std::fs::remove_file(root.join("delete")).unwrap();
    std::fs::write(root.join("new"), "untracked").unwrap();
    std::fs::write(root.join("added"), "staged").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(Path::new("added")).unwrap();
    index.write().unwrap();
    let index_before = std::fs::read(repo.path().join("index")).unwrap();
    let config_before = std::fs::read(repo.path().join("config")).unwrap();
    let head_before = std::fs::read(repo.path().join("HEAD")).unwrap();
    let r = s.git_connect(&p, &root).unwrap();
    let overview = s.git_overview(&p, &r.id, 0, 1).unwrap();
    assert_eq!(overview.commits[0].hash, first.to_string());
    assert_eq!(overview.commits[0].author, "Local Author");
    assert_eq!(
        overview.commits[0].committed_at,
        "2023-11-14T22:13:20+00:00"
    );
    let status = overview.status.unwrap();
    assert_eq!(
        (
            status.modified,
            status.added,
            status.deleted,
            status.untracked
        ),
        (1, 1, 1, 1)
    );
    let branches = s.git_branches(&p, &r.id, "F1-1", 0, 25).unwrap();
    assert_eq!(branches.branches.len(), 1);
    assert_eq!(branches.branches[0].name, branch);
    assert!(s
        .git_branches(&p, &r.id, "none", 0, 25)
        .unwrap()
        .branches
        .is_empty());
    assert_eq!(
        std::fs::read(repo.path().join("index")).unwrap(),
        index_before
    );
    assert_eq!(
        std::fs::read(repo.path().join("config")).unwrap(),
        config_before
    );
    assert_eq!(
        std::fs::read(repo.path().join("HEAD")).unwrap(),
        head_before
    );
    assert_eq!(
        std::fs::read_to_string(root.join("tracked")).unwrap(),
        "changed"
    );
    assert!(!repo.path().join("index.lock").exists());
    // NO_SEARCH does not accidentally connect a parent repository.
    std::fs::create_dir(root.join("nested")).unwrap();
    assert!(s.git_connect(&p, &root.join("nested")).is_err());
}
#[test]
fn worktree_bare_unborn_detached_and_bounded_pages() {
    let (dir, mut s, p, _) = setup();
    let empty = Repository::init(dir.path().join("empty")).unwrap();
    let r = s.git_connect(&p, empty.workdir().unwrap()).unwrap();
    let overview = s.git_overview(&p, &r.id, 0, 20).unwrap();
    assert!(overview.branch.contains("unborn"));
    assert!(overview.commits.is_empty());
    let (repo, first) = fixture(&dir.path().join("repo"));
    commit(&repo, "Second");
    let root = dir.path().join("worktree");
    repo.worktree("linked", &root, None).unwrap();
    let r = s.git_connect(&p, &root).unwrap();
    assert_eq!(s.git_overview(&p, &r.id, 0, 20).unwrap().branch, "linked");
    let r = s.git_connect(&p, repo.workdir().unwrap()).unwrap();
    assert!(s.git_overview(&p, &r.id, 0, 1).unwrap().has_more);
    assert_eq!(
        s.git_overview(&p, &r.id, 1, 1).unwrap().commits[0].hash,
        first.to_string()
    );
    assert!(s.git_overview(&p, &r.id, 0, 101).is_err());
    assert!(s.git_branches(&p, &r.id, "", 10001, 1).is_err());
    let page = s.git_branches(&p, &r.id, "", 0, 1).unwrap();
    assert!(page.has_more);
    repo.set_head_detached(first).unwrap();
    assert!(s
        .git_overview(&p, &r.id, 0, 1)
        .unwrap()
        .branch
        .contains("Detached"));
    let bare = Repository::clone(
        repo.workdir().unwrap().to_str().unwrap(),
        dir.path().join("clone"),
    )
    .unwrap();
    // A real bare clone; local file transport only in fixture construction.
    let mut builder = git2::build::RepoBuilder::new();
    builder.bare(true);
    let bare = builder
        .clone(
            bare.workdir().unwrap().to_str().unwrap(),
            &dir.path().join("bare.git"),
        )
        .unwrap();
    let r = s.git_connect(&p, bare.path()).unwrap();
    let overview = s.git_overview(&p, &r.id, 0, 20).unwrap();
    assert!(overview.bare);
    assert!(overview.status.is_none());
    assert!(!overview.commits.is_empty());
}
#[test]
fn links_scoped_confirmed_persisted_historical_and_cascading() {
    let (dir, mut s, p, issue) = setup();
    let (repo, oid) = fixture(&dir.path().join("repo"));
    repo.branch("feature/F1-1", &repo.find_commit(oid).unwrap(), false)
        .unwrap();
    let r = s.git_connect(&p, repo.workdir().unwrap()).unwrap();
    let mut data = input(&p, &issue, &r, "branch", "feature/F1-1");
    data.confirmed = false;
    assert!(s.git_link(data).is_err());
    s.git_link(input(&p, &issue, &r, "branch", "feature/F1-1"))
        .unwrap();
    let links = s
        .git_link(input(&p, &issue, &r, "commit", &oid.to_string()))
        .unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(
        s.git_link(input(&p, &issue, &r, "commit", &oid.to_string()))
            .unwrap()
            .len(),
        2
    );
    for (kind, target) in [
        ("branch", "../bad"),
        ("commit", "HEAD"),
        ("commit", "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
        ("shell", "echo"),
    ] {
        assert!(s.git_link(input(&p, &issue, &r, kind, target)).is_err());
    }
    repo.find_branch("feature/F1-1", git2::BranchType::Local)
        .unwrap()
        .rename("renamed", false)
        .unwrap();
    repo.find_branch("renamed", git2::BranchType::Local)
        .unwrap()
        .delete()
        .unwrap();
    drop(s);
    let mut s = Service::open(&dir.path().join("forma.sqlite3")).unwrap();
    assert_eq!(s.git_links(&p, &issue.id).unwrap().len(), 2);
    assert!(s.git_overview(&p, &r.id, 0, 20).is_ok());
    s.git_disconnect(&p).unwrap();
    assert!(!s.git_links(&p, &issue.id).unwrap()[0].connected);
    assert!(s.git_overview(&p, &r.id, 0, 20).is_err());
    s.git_connect(&p, repo.workdir().unwrap()).unwrap();
    let other = s
        .save_project(
            None,
            ProjectInput {
                workspace_id: s.snapshot().unwrap().active_workspace_id.unwrap(),
                name: "Other".into(),
                description: "".into(),
                icon: "code".into(),
                color: "#123456".into(),
                status: "active".into(),
            },
        )
        .unwrap()
        .projects
        .into_iter()
        .find(|x| x.id != p)
        .unwrap()
        .id;
    assert!(s.git_links(&other, &issue.id).is_err());
    assert!(s.git_overview(&other, &r.id, 0, 1).is_err());
    let link = s.git_links(&p, &issue.id).unwrap().remove(0);
    s.git_unlink(&p, &issue.id, &link.id).unwrap();
    assert_eq!(s.git_links(&p, &issue.id).unwrap().len(), 1);
    s.delete_issue(&p, &issue.id).unwrap();
    assert_eq!(
        s.repo
            .conn
            .query_row("SELECT count(*) FROM git_links", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        0
    );
    s.delete_project(&p).unwrap();
    assert_eq!(
        s.repo
            .conn
            .query_row("SELECT count(*) FROM git_repositories", [], |r| r
                .get::<_, i64>(0))
            .unwrap(),
        0
    );
}
#[test]
fn backup_restore_paths_require_native_reauthorization_even_after_restart() {
    let (dir, mut s, p, issue) = setup();
    let root = dir.path().join("repo");
    let (repo, oid) = fixture(&root);
    let r = s.git_connect(&p, &root).unwrap();
    s.git_link(input(&p, &issue, &r, "commit", &oid.to_string()))
        .unwrap();
    let backup = dir.path().join("backup.forma-backup");
    s.export_backup(&backup).unwrap();
    drop(repo);
    std::fs::remove_dir_all(&root).unwrap();
    s.prepare_restore(&backup).unwrap();
    s.restore_backup(true).unwrap();
    let connection = s.git_connection(&p).unwrap().unwrap();
    assert!(!connection.trusted);
    assert!(s
        .git_overview(&p, &r.id, 0, 20)
        .unwrap_err()
        .message
        .contains("Select the repository"));
    assert_eq!(s.git_links(&p, &issue.id).unwrap().len(), 1);
    drop(s);
    let mut s = Service::open(&dir.path().join("forma.sqlite3")).unwrap();
    assert!(!s.git_connection(&p).unwrap().unwrap().trusted);
    assert!(s.git_connect(&p, &root).is_err());
    fixture(&root);
    assert!(s.git_connect(&p, &root).unwrap().trusted);
}
#[test]
fn missing_and_corrupt_repository_fail_without_losing_history() {
    let (dir, mut s, p, issue) = setup();
    let root = dir.path().join("repo");
    let (repo, oid) = fixture(&root);
    let r = s.git_connect(&p, &root).unwrap();
    s.git_link(input(&p, &issue, &r, "commit", &oid.to_string()))
        .unwrap();
    drop(repo);
    std::fs::rename(&root, dir.path().join("moved")).unwrap();
    assert_eq!(s.git_overview(&p, &r.id, 0, 20).unwrap_err().code, "git");
    std::fs::rename(dir.path().join("moved"), &root).unwrap();
    std::fs::write(root.join(".git/index"), b"corrupted").unwrap();
    assert!(s.git_overview(&p, &r.id, 0, 20).is_err());
    assert_eq!(s.git_links(&p, &issue.id).unwrap().len(), 1);
}
#[test]
fn populated_schema3_upgrade_and_m03_backup_compatibility() {
    let (dir, mut s, p, issue) = setup();
    let legacy = dir.path().join("m03.forma-backup");
    s.export_backup(&legacy).unwrap();
    let c = rusqlite::Connection::open(&legacy).unwrap();
    c.execute_batch("DROP TRIGGER git_link_scope; DROP TRIGGER git_link_scope_update; DROP TABLE git_links; DROP TABLE git_repositories; PRAGMA user_version=3; UPDATE backup_metadata SET format_version=1;").unwrap();
    drop(c);
    s.prepare_restore(&legacy).unwrap();
    s.restore_backup(true).unwrap();
    assert_eq!(s.issue(&p, &issue.id).unwrap().identifier, issue.identifier);
    assert!(s.git_connection(&p).unwrap().is_none());
    let live = dir.path().join("upgrade.sqlite3");
    std::fs::copy(&legacy, &live).unwrap();
    let s = Service::open(&live).unwrap();
    assert_eq!(s.issue(&p, &issue.id).unwrap().title, issue.title);
    assert_eq!(
        s.repo
            .conn
            .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        4
    );
}
#[test]
#[ignore = "manual real Git performance measurement"]
fn git_benchmark() {
    let (dir, mut s, p, _) = setup();
    let root = dir.path().join("repo");
    let (repo, oid) = fixture(&root);
    for i in 0..500 {
        commit(&repo, &format!("Commit {i}"));
    }
    for i in 0..1000 {
        repo.branch(
            &format!("feature/{i:04}"),
            &repo.find_commit(oid).unwrap(),
            false,
        )
        .unwrap();
    }
    for i in 0..10000 {
        std::fs::write(root.join(format!("untracked-{i}")), "data").unwrap();
    }
    let start = std::time::Instant::now();
    let r = s.git_connect(&p, &root).unwrap();
    println!(
        "connect (501 commits, 1001 branches, 10000 untracked files): {:?}",
        start.elapsed()
    );
    for _ in 0..5 {
        let start = std::time::Instant::now();
        let overview = s.git_overview(&p, &r.id, 0, 20).unwrap();
        println!(
            "overview/status: {:?}, untracked={}",
            start.elapsed(),
            overview.status.unwrap().untracked
        );
        let start = std::time::Instant::now();
        s.git_branches(&p, &r.id, "099", 0, 25).unwrap();
        println!("filtered branches: {:?}", start.elapsed());
    }
}

#[test]
fn failed_004_migration_rolls_back_and_retries_without_data_loss() {
    let (dir, s, p, issue) = setup();
    let legacy = dir.path().join("upgrade-failure.sqlite3");
    s.export_backup(&legacy).unwrap();
    let c = rusqlite::Connection::open(&legacy).unwrap();
    c.execute_batch("DROP TRIGGER git_link_scope; DROP TRIGGER git_link_scope_update; DROP TABLE git_links; DROP TABLE git_repositories; PRAGMA user_version=3; CREATE TABLE git_links(collision);").unwrap();
    drop(c);
    assert!(Service::open(&legacy).is_err());
    let c = rusqlite::Connection::open(&legacy).unwrap();
    assert_eq!(
        c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert!(!c
        .prepare("SELECT 1 FROM sqlite_master WHERE name='git_repositories'")
        .unwrap()
        .exists([])
        .unwrap());
    c.execute_batch("DROP TABLE git_links").unwrap();
    drop(c);
    let s = Service::open(&legacy).unwrap();
    assert_eq!(s.issue(&p, &issue.id).unwrap().identifier, issue.identifier);
}

#[cfg(unix)]
#[test]
fn reads_do_not_execute_repository_hooks_filters_or_fsmonitor() {
    use std::os::unix::fs::PermissionsExt;
    let (dir, mut s, p, issue) = setup();
    let root = dir.path().join("repo");
    let (repo, oid) = fixture(&root);
    let script = repo.path().join("hooks/post-checkout");
    let marker = dir.path().join("executed");
    std::fs::write(
        &script,
        format!("#!/bin/sh\necho executed > '{}'\n", marker.display()),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let mut config = repo.config().unwrap();
    config
        .set_str("core.fsmonitor", script.to_str().unwrap())
        .unwrap();
    config
        .set_str("filter.untrusted.clean", script.to_str().unwrap())
        .unwrap();
    config
        .set_str("filter.untrusted.smudge", script.to_str().unwrap())
        .unwrap();
    std::fs::write(root.join(".gitattributes"), "tracked filter=untrusted\n").unwrap();
    std::fs::write(root.join("tracked"), "changed\n").unwrap();
    let r = s.git_connect(&p, &root).unwrap();
    s.git_overview(&p, &r.id, 0, 20).unwrap();
    s.git_branches(&p, &r.id, "", 0, 25).unwrap();
    s.git_link(input(&p, &issue, &r, "commit", &oid.to_string()))
        .unwrap();
    assert!(!marker.exists());
}
