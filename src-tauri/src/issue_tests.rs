use crate::{domain::*, service::Service};
fn setup() -> (tempfile::TempDir, Service, String, String) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Service::open(&d.path().join("test.sqlite3")).unwrap();
    let w = s
        .create_workspace("Studio")
        .unwrap()
        .active_workspace_id
        .unwrap();
    let p = project(&mut s, &w);
    (d, s, w, p)
}
fn project(s: &mut Service, w: &str) -> String {
    s.save_project(
        None,
        ProjectInput {
            workspace_id: w.into(),
            name: "Engine".into(),
            description: "".into(),
            icon: "code".into(),
            color: "#112233".into(),
            status: "active".into(),
        },
    )
    .unwrap()
    .projects[0]
        .id
        .clone()
}
fn input(p: &str) -> IssueInput {
    IssueInput {
        project_id: p.into(),
        title: "  Ship engine  ".into(),
        description: "# Details\nPrivate description".into(),
        status: "todo".into(),
        priority: "high".into(),
        due_date: Some("2028-02-29".into()),
        parent_id: None,
        label_ids: vec![],
        position: 10,
    }
}
fn query(p: &str) -> IssueQuery {
    IssueQuery {
        label_id: None,
        due_from: None,
        due_to: None,
        undated: false,
        project_id: p.into(),
        search: "".into(),
        status: None,
        priority: None,
        archived: false,
        sort: "position".into(),
        offset: 0,
        limit: 25,
    }
}
fn label(s: &mut Service, w: &str) -> Label {
    s.save_label(
        None,
        LabelInput {
            workspace_id: w.into(),
            name: "Bug".into(),
            color: "#ff0000".into(),
        },
    )
    .unwrap()
    .remove(0)
}
#[test]
fn populated_m01_upgrade_preserves_data_assignments_and_is_idempotent() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test.sqlite3");
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    let w = uuid::Uuid::new_v4().to_string();
    let p = uuid::Uuid::new_v4().to_string();
    c.execute(
        "INSERT INTO workspaces VALUES(?1,'Studio','2026','2026')",
        [&w],
    )
    .unwrap();
    c.execute("INSERT INTO projects VALUES(?1,?2,'Existing','Keep me','code','#123456','active',0,'2026','2026')",rusqlite::params![p,w]).unwrap();
    c.execute("UPDATE app_state SET active_workspace_id=?1", [&w])
        .unwrap();
    for key in ["a", "b"] {
        c.execute(
            "INSERT INTO issues VALUES(?1,?2,'Legacy','# Markdown','backlog','2026','2026')",
            rusqlite::params![key, p],
        )
        .unwrap();
    }
    c.execute(
        "INSERT INTO labels VALUES('label',?1,'Legacy','#123456','2026','2026')",
        [&w],
    )
    .unwrap();
    c.execute("INSERT INTO issue_labels VALUES('a','label','2026')", [])
        .unwrap();
    c.execute(
        "INSERT INTO activity_events VALUES('event',?1,?2,'old','Retain','2026')",
        rusqlite::params![w, p],
    )
    .unwrap();
    c.pragma_update(None, "user_version", 1).unwrap();
    drop(c);
    let mut s = Service::open(&path).unwrap();
    assert_eq!(s.snapshot().unwrap().active_workspace_id, Some(w));
    assert_eq!(s.snapshot().unwrap().projects[0].description, "Keep me");
    let rows = s.issues(query(&p)).unwrap().issues;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].id, "a");
    assert_eq!(rows[0].number, 1);
    assert_eq!(rows[1].number, 2);
    assert_eq!(rows[0].label_ids, vec!["label"]);
    assert_eq!(rows[0].description, "# Markdown");
    assert_eq!(s.save_issue(None, input(&p)).unwrap().number, 3);
    assert_eq!(
        s.repo
            .conn
            .query_row(
                "SELECT summary FROM activity_events WHERE id='event'",
                [],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "Retain"
    );
    drop(s);
    let s = Service::open(&path).unwrap();
    assert_eq!(s.issues(query(&p)).unwrap().total, 3);
    assert_eq!(
        s.repo
            .conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
    assert!(!s
        .repo
        .conn
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
}
#[test]
fn failed_upgrade_rolls_back_all_002_changes_and_version() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("test.sqlite3");
    let c = rusqlite::Connection::open(&path).unwrap();
    c.execute_batch(include_str!("../migrations/001_initial.sql"))
        .unwrap();
    c.execute_batch("CREATE TABLE project_keys(number); PRAGMA user_version=1;")
        .unwrap();
    drop(c);
    assert!(Service::open(&path).is_err());
    let c = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    assert!(c.prepare("SELECT issue_sequence FROM projects").is_err());
    c.execute("DROP TABLE project_keys", []).unwrap();
    drop(c);
    assert!(Service::open(&path).is_ok());
}
#[test]
fn issue_crud_all_fields_activity_archive_and_restart() {
    let (d, mut s, w, p) = setup();
    let l = label(&mut s, &w);
    let mut i = input(&p);
    i.label_ids = vec![l.id.clone()];
    let created = s.save_issue(None, i.clone()).unwrap();
    assert_eq!(created.title, "Ship engine");
    assert_eq!(created.identifier, "F1-1");
    i.title = "Revised".into();
    i.description = "Updated".into();
    i.status = "in_progress".into();
    i.priority = "urgent".into();
    i.due_date = None;
    i.position = 4;
    i.label_ids.clear();
    let edited = s.save_issue(Some(&created.id), i.clone()).unwrap();
    assert_eq!(edited.identifier, created.identifier);
    assert_eq!(edited.created_at, created.created_at);
    assert_eq!(edited.description, "Updated");
    assert_eq!(edited.priority, "urgent");
    assert_eq!(edited.status, "in_progress");
    assert_eq!(edited.due_date, None);
    assert_eq!(edited.position, 4);
    assert!(edited.label_ids.is_empty());
    let a = s.issue_activity(&p, &created.id, 0, 1).unwrap();
    assert_eq!(a.total, 2);
    assert_eq!(a.events.len(), 1);
    assert!(!a.events[0].summary.contains("Revised"));
    s.save_issue(Some(&created.id), i).unwrap();
    assert_eq!(s.issue_activity(&p, &created.id, 0, 100).unwrap().total, 2);
    s.archive_issue(&p, &created.id, true).unwrap();
    assert_eq!(s.issues(query(&p)).unwrap().total, 0);
    let mut q = query(&p);
    q.archived = true;
    assert_eq!(s.issues(q).unwrap().total, 1);
    s.archive_issue(&p, &created.id, false).unwrap();
    drop(s);
    let mut s = Service::open(&d.path().join("test.sqlite3")).unwrap();
    assert_eq!(s.issue(&p, &created.id).unwrap().title, "Revised");
    assert_eq!(s.labels(&w).unwrap()[0].id, l.id);
    assert_eq!(
        s.issue_activity(&p, &created.id, 1, 1)
            .unwrap()
            .events
            .len(),
        1
    );
    s.delete_issue(&p, &created.id).unwrap();
    assert!(s.issue(&p, &created.id).is_err());
    assert_eq!(s.save_issue(None, input(&p)).unwrap().number, 2);
}
#[test]
fn numbering_is_project_scoped_and_never_reuses_deleted_numbers() {
    let (_d, mut s, w, p) = setup();
    let p2 = project(&mut s, &w);
    let a = s.save_issue(None, input(&p)).unwrap();
    let b = s.save_issue(None, input(&p2)).unwrap();
    assert_eq!(a.number, b.number);
    assert_ne!(a.identifier, b.identifier);
    s.delete_issue(&p, &a.id).unwrap();
    assert_eq!(s.save_issue(None, input(&p)).unwrap().number, 2);
    s.delete_project(&p).unwrap();
    let p3 = project(&mut s, &w);
    assert_ne!(
        s.save_issue(None, input(&p3)).unwrap().identifier,
        a.identifier
    );
}
#[test]
fn scope_and_hierarchy_reject_cycles_deep_nesting_and_wrong_projects() {
    let (_d, mut s, w, p) = setup();
    let parent = s.save_issue(None, input(&p)).unwrap();
    let mut i = input(&p);
    i.parent_id = Some(parent.id.clone());
    let child = s.save_issue(None, i.clone()).unwrap();
    assert_eq!(
        s.subtasks(&p, &parent.id, 0, 1).unwrap().issues[0].id,
        child.id
    );
    i.parent_id = Some(child.id.clone());
    assert!(s.save_issue(None, i.clone()).is_err());
    assert!(s.save_issue(Some(&parent.id), i.clone()).is_err());
    i.parent_id = Some(parent.id.clone());
    assert!(s.save_issue(Some(&parent.id), i.clone()).is_err());
    let p2 = project(&mut s, &w);
    i.project_id = p2.clone();
    assert!(s.save_issue(None, i).is_err());
    assert!(s.issue(&p2, &parent.id).is_err());
    assert!(s.delete_issue(&p2, &parent.id).is_err());
    assert!(s.archive_issue(&p2, &parent.id, true).is_err());
    assert!(s.issue_activity(&p2, &parent.id, 0, 20).is_err());
    let w2 = s
        .create_workspace("Other")
        .unwrap()
        .active_workspace_id
        .unwrap();
    let p3 = project(&mut s, &w2);
    assert_eq!(s.issues(query(&p3)).unwrap().total, 0);
    let mut i = input(&p3);
    assert!(s.save_issue(Some(&parent.id), i.clone()).is_err());
    i.parent_id = Some(parent.id.clone());
    assert!(s.save_issue(None, i).is_err());
    s.delete_issue(&p, &parent.id).unwrap();
    assert_eq!(s.issue(&p, &child.id).unwrap().parent_id, None);
    assert!(s
        .issue_activity(&p, &child.id, 0, 20)
        .unwrap()
        .events
        .iter()
        .any(|e| e.summary.contains("Parent deleted")));
}
#[test]
fn labels_validate_scope_duplicates_and_delete_assignments() {
    let (_d, mut s, w, p) = setup();
    let l = label(&mut s, &w);
    let w2 = s
        .create_workspace("Other")
        .unwrap()
        .active_workspace_id
        .unwrap();
    let l2 = label(&mut s, &w2);
    let mut i = input(&p);
    i.label_ids = vec![l2.id.clone()];
    assert!(s.save_issue(None, i.clone()).is_err());
    i.label_ids = vec![l.id.clone(), l.id.clone()];
    assert!(s.save_issue(None, i.clone()).is_err());
    i.label_ids = vec![l.id.clone()];
    let issue = s.save_issue(None, i).unwrap();
    assert!(s
        .save_label(
            None,
            LabelInput {
                workspace_id: w.clone(),
                name: " bug ".into(),
                color: "#123456".into()
            }
        )
        .is_err());
    assert!(s
        .save_label(
            Some(&l.id),
            LabelInput {
                workspace_id: w2.clone(),
                name: "Moved".into(),
                color: "#123456".into()
            }
        )
        .is_err());
    assert!(s.delete_label(&w2, &l.id).is_err());
    let labels = s
        .save_label(
            Some(&l.id),
            LabelInput {
                workspace_id: w.clone(),
                name: "Defect".into(),
                color: "#112233".into(),
            },
        )
        .unwrap();
    assert_eq!(labels[0].name, "Defect");
    assert!(s
        .repo
        .conn
        .execute(
            "INSERT INTO issue_labels VALUES(?1,?2,'now')",
            rusqlite::params![issue.id, l2.id]
        )
        .is_err());
    s.delete_label(&w, &l.id).unwrap();
    assert!(s.issue(&p, &issue.id).unwrap().label_ids.is_empty());
    assert_eq!(s.issue_activity(&p, &issue.id, 0, 20).unwrap().total, 2);
}
#[test]
fn input_validation_and_pagination_reject_before_writes() {
    let (_d, mut s, _w, p) = setup();
    for field in 0..11 {
        let mut i = input(&p);
        match field {
            0 => i.title = "\n".into(),
            1 => i.title = "x".repeat(201),
            2 => i.description = "x".repeat(50001),
            3 => i.status = "bad".into(),
            4 => i.priority = "bad".into(),
            5 => i.due_date = Some("2027-02-29".into()),
            6 => i.due_date = Some("not a date".into()),
            7 => i.position = -1,
            8 => i.parent_id = Some("bad".into()),
            9 => i.label_ids = vec!["bad".into()],
            _ => i.project_id = "bad".into(),
        };
        assert_eq!(s.save_issue(None, i).unwrap_err().code, "validation");
    }
    assert_eq!(s.issues(query(&p)).unwrap().total, 0);
    for field in 0..5 {
        let mut q = query(&p);
        match field {
            0 => q.limit = 101,
            1 => q.offset = -1,
            2 => q.sort = "bad".into(),
            3 => q.status = Some("bad".into()),
            _ => q.priority = Some("bad".into()),
        };
        assert!(s.issues(q).is_err());
    }
}
#[test]
fn filters_counts_sort_and_pagination_are_deterministic() {
    let (_d, mut s, _w, p) = setup();
    let mut i = input(&p);
    i.title = "Alpha".into();
    i.position = 20;
    i.priority = "low".into();
    let a = s.save_issue(None, i.clone()).unwrap();
    i.title = "Beta".into();
    i.position = 1;
    i.status = "done".into();
    i.priority = "urgent".into();
    i.due_date = None;
    let b = s.save_issue(None, i).unwrap();
    let mut q = query(&p);
    q.limit = 1;
    let page = s.issues(q.clone()).unwrap();
    assert_eq!(page.total, 2);
    assert_eq!(page.done, 1);
    assert_eq!(page.issues[0].id, b.id);
    q.offset = 1;
    assert_eq!(s.issues(q.clone()).unwrap().issues[0].id, a.id);
    q.offset = 0;
    q.sort = "priority".into();
    assert_eq!(s.issues(q.clone()).unwrap().issues[0].id, b.id);
    q.sort = "due".into();
    assert_eq!(s.issues(q.clone()).unwrap().issues[0].id, a.id);
    q.status = Some("done".into());
    assert_eq!(s.issues(q.clone()).unwrap().total, 1);
    q.priority = Some("low".into());
    assert_eq!(s.issues(q.clone()).unwrap().total, 0);
    q.status = None;
    q.priority = None;
    q.search = a.identifier.to_lowercase();
    assert_eq!(s.issues(q.clone()).unwrap().issues[0].id, a.id);
    q.search = "alpha".into();
    assert_eq!(s.issues(q.clone()).unwrap().total, 1);
    q.search = "%".into();
    assert_eq!(s.issues(q).unwrap().total, 0);
    for sort in ["position", "priority", "newest", "updated", "due"] {
        let mut q = query(&p);
        q.sort = sort.into();
        assert_eq!(
            s.issues(q.clone())
                .unwrap()
                .issues
                .iter()
                .map(|i| &i.id)
                .collect::<Vec<_>>(),
            s.issues(q)
                .unwrap()
                .issues
                .iter()
                .map(|i| &i.id)
                .collect::<Vec<_>>()
        );
    }
}
#[test]
fn failed_activity_rolls_back_issue_labels_numbering_archive_and_deletion() {
    let (_d, mut s, w, p) = setup();
    let l = label(&mut s, &w);
    let first = s.save_issue(None, input(&p)).unwrap();
    s.repo.conn.execute_batch("CREATE TRIGGER reject_issue_activity BEFORE INSERT ON activity_events BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    let mut i = input(&p);
    i.label_ids = vec![l.id.clone()];
    i.title = "Changed".into();
    assert!(s.save_issue(None, i.clone()).is_err());
    assert!(s.save_issue(Some(&first.id), i).is_err());
    assert!(s.archive_issue(&p, &first.id, true).is_err());
    assert!(s.delete_issue(&p, &first.id).is_err());
    assert!(s.delete_label(&w, &l.id).is_err());
    let original = s.issue(&p, &first.id).unwrap();
    assert_eq!(original.title, "Ship engine");
    assert!(!original.archived);
    assert!(original.label_ids.is_empty());
    assert_eq!(s.issues(query(&p)).unwrap().total, 1);
    assert_eq!(s.labels(&w).unwrap().len(), 1);
    s.repo
        .conn
        .execute("DROP TRIGGER reject_issue_activity", [])
        .unwrap();
    assert_eq!(s.save_issue(None, input(&p)).unwrap().number, 2);
}

#[test]
fn archived_parent_keeps_existing_children_editable_and_constraints_reject_invalid_rows() {
    let (_d, mut s, _w, p) = setup();
    let parent = s.save_issue(None, input(&p)).unwrap();
    let mut i = input(&p);
    i.parent_id = Some(parent.id.clone());
    let child = s.save_issue(None, i.clone()).unwrap();
    s.archive_issue(&p, &parent.id, true).unwrap();
    i.title = "Still editable".into();
    assert!(s.save_issue(Some(&child.id), i.clone()).is_ok());
    assert!(s.save_issue(None, i).is_err());
    let c = &s.repo.conn;
    assert!(c
        .execute(
            "UPDATE issues SET parent_id=?1 WHERE id=?2",
            rusqlite::params![child.id, parent.id]
        )
        .is_err());
    for sql in [
        "UPDATE issues SET status='invalid'",
        "UPDATE issues SET priority='invalid'",
        "UPDATE issues SET number=0",
        "UPDATE issues SET title=''",
        "UPDATE issues SET position=-1",
        "UPDATE issues SET archived=2",
        "UPDATE issues SET due_date='bad'",
    ] {
        assert!(c.execute(sql, []).is_err(), "{sql}");
    }
    assert_eq!(s.issue(&p, &child.id).unwrap().title, "Still editable");
}

#[test]
fn board_moves_are_atomic_scoped_dense_and_survive_restart() {
    let (dir, mut s, w, p) = setup();
    let other = project(&mut s, &w);
    let a = s.save_issue(None, input(&p)).unwrap();
    let b = s.save_issue(None, input(&p)).unwrap();
    let c = s.save_issue(None, input(&p)).unwrap();
    let foreign = s.save_issue(None, input(&other)).unwrap();
    s.move_issue(&p, &c.id, "todo", Some(&a.id)).unwrap();
    let page = s.issues(query(&p)).unwrap();
    assert_eq!(
        page.issues
            .iter()
            .map(|i| i.id.as_str())
            .collect::<Vec<_>>(),
        vec![c.id.as_str(), a.id.as_str(), b.id.as_str()]
    );
    assert_eq!(
        page.issues.iter().map(|i| i.position).collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    s.move_issue(&p, &a.id, "done", None).unwrap();
    s.move_issue(&p, &b.id, "done", Some(&a.id)).unwrap();
    assert!(s.move_issue(&p, &c.id, "done", Some(&foreign.id)).is_err());
    assert!(s.move_issue(&other, &c.id, "todo", None).is_err());
    s.repo.conn.execute_batch("CREATE TRIGGER reject_move BEFORE INSERT ON activity_events WHEN NEW.kind='issue.moved' BEGIN SELECT RAISE(ABORT,'injected failure'); END;").unwrap();
    let before = serde_json::to_value(s.issues(query(&p)).unwrap()).unwrap();
    assert!(s.move_issue(&p, &c.id, "done", Some(&a.id)).is_err());
    assert_eq!(
        before,
        serde_json::to_value(s.issues(query(&p)).unwrap()).unwrap()
    );
    s.repo
        .conn
        .execute_batch("DROP TRIGGER reject_move")
        .unwrap();
    s.set_project_view(&p, "board").unwrap();
    drop(s);
    let s = Service::open(&dir.path().join("test.sqlite3")).unwrap();
    assert_eq!(s.project_view(&p).unwrap(), "board");
    assert_eq!(s.issue(&p, &a.id).unwrap().status, "done");
    let mut q = query(&p);
    q.status = Some("done".into());
    let issues = s.issues(q).unwrap().issues;
    assert_eq!(
        issues.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        vec![b.id.as_str(), a.id.as_str()]
    );
    assert_eq!(
        issues.iter().map(|i| i.position).collect::<Vec<_>>(),
        vec![0, 1]
    );
}

#[test]
fn concurrent_connections_move_without_lost_or_duplicate_cards() {
    let (dir, mut s, _, p) = setup();
    let ids = (0..12)
        .map(|_| s.save_issue(None, input(&p)).unwrap().id)
        .collect::<Vec<_>>();
    let handles = (0..4)
        .map(|thread| {
            let path = dir.path().join("test.sqlite3");
            let p = p.clone();
            let ids = ids.clone();
            std::thread::spawn(move || {
                let mut s = Service::open(&path).unwrap();
                for turn in 0..20 {
                    let id = &ids[(turn + thread) % ids.len()];
                    s.move_issue(&p, id, if turn % 2 == 0 { "done" } else { "todo" }, None)
                        .unwrap();
                }
            })
        })
        .collect::<Vec<_>>();
    for h in handles {
        h.join().unwrap();
    }
    let all = s.issues(query(&p)).unwrap();
    assert_eq!(all.issues.len(), 12);
    for status in ["todo", "done"] {
        let mut q = query(&p);
        q.status = Some(status.into());
        let issues = s.issues(q).unwrap().issues;
        assert_eq!(
            issues.iter().map(|i| i.position).collect::<Vec<_>>(),
            (0..issues.len() as i64).collect::<Vec<_>>()
        );
    }
    assert!(!s
        .repo
        .conn
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
}

#[test]
fn combined_label_date_text_filters_and_subtask_progress() {
    let (_, mut s, w, p) = setup();
    let l = label(&mut s, &w);
    let mut data = input(&p);
    data.label_ids = vec![l.id.clone()];
    let parent = s.save_issue(None, data.clone()).unwrap();
    data.parent_id = Some(parent.id.clone());
    data.status = "done".into();
    data.due_date = None;
    s.save_issue(None, data).unwrap();
    s.save_issue(None, input(&p)).unwrap();
    let mut q = query(&p);
    q.label_id = Some(l.id);
    q.due_from = Some("2028-02-01".into());
    q.due_to = Some("2028-02-29".into());
    q.search = "ship".into();
    q.status = Some("todo".into());
    q.priority = Some("high".into());
    let page = s.issues(q.clone()).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.issues[0].id, parent.id);
    assert_eq!(
        (page.issues[0].subtask_done, page.issues[0].subtask_total),
        (1, 1)
    );
    q.undated = true;
    assert!(s.issues(q.clone()).is_err());
    q.due_from = None;
    q.due_to = None;
    q.status = None;
    assert_eq!(s.issues(q.clone()).unwrap().total, 1);
    q.due_from = Some("2028-02-30".into());
    assert!(s.issues(q).is_err());
}

#[test]
fn backup_roundtrip_all_tables_private_staging_recovery_and_restart() {
    let (dir, mut s, w, p) = setup();
    let l = label(&mut s, &w);
    let mut data = input(&p);
    data.label_ids = vec![l.id];
    let parent = s.save_issue(None, data.clone()).unwrap();
    data.parent_id = Some(parent.id.clone());
    let child = s.save_issue(None, data).unwrap();
    s.move_issue(&p, &child.id, "done", None).unwrap();
    s.set_project_view(&p, "board").unwrap();
    s.set_theme("dark").unwrap();
    let backup = dir.path().join("export.forma-backup");
    s.export_backup(&backup).unwrap();
    let expected = serde_json::to_value(s.snapshot().unwrap()).unwrap();
    s.prepare_restore(&backup).unwrap();
    assert!(s.restore_backup(false).is_err());
    // Replacing/removing the selected file after validation cannot change staged data.
    std::fs::write(&backup, b"corrupted after selection").unwrap();
    s.delete_workspace(&w).unwrap();
    let recovery = s.restore_backup(true).unwrap();
    assert!(std::path::Path::new(&recovery).exists());
    assert_eq!(
        expected,
        serde_json::to_value(s.snapshot().unwrap()).unwrap()
    );
    assert_eq!(s.issue(&p, &child.id).unwrap().parent_id, Some(parent.id));
    assert_eq!(s.project_view(&p).unwrap(), "board");
    assert_eq!(s.labels(&w).unwrap().len(), 1);
    assert!(s.issue_activity(&p, &child.id, 0, 100).unwrap().total >= 2);
    drop(s);
    let mut s = Service::open(&dir.path().join("test.sqlite3")).unwrap();
    assert_eq!(
        expected,
        serde_json::to_value(s.snapshot().unwrap()).unwrap()
    );
    s.prepare_restore(std::path::Path::new(&recovery)).unwrap();
    s.restore_backup(true).unwrap();
    assert!(s.snapshot().unwrap().workspaces.is_empty());
}

#[test]
fn backup_rejects_corruption_versions_schema_and_foreign_keys_without_replacing_live_data() {
    let (dir, mut s, _, _) = setup();
    let before = serde_json::to_value(s.snapshot().unwrap()).unwrap();
    for damage in [
        "garbage",
        "version",
        "format",
        "foreign_key",
        "schema",
        "truncated",
    ] {
        let path = dir.path().join(format!("{damage}.forma-backup"));
        s.export_backup(&path).unwrap();
        match damage {
            "garbage" => std::fs::write(&path, b"not sqlite").unwrap(),
            "truncated" => std::fs::OpenOptions::new()
                .write(true)
                .open(&path)
                .unwrap()
                .set_len(180)
                .unwrap(),
            _ => {
                let c = rusqlite::Connection::open(&path).unwrap();
                match damage {
                    "version" => c.pragma_update(None,"user_version",999).unwrap(),
                    "format" => {c.execute("UPDATE backup_metadata SET format_version=99",[]).unwrap();},
                    "foreign_key" => {c.execute_batch("PRAGMA foreign_keys=OFF; UPDATE app_state SET active_workspace_id='missing';").unwrap();},
                    _ => c.execute_batch("CREATE TRIGGER unexpected AFTER INSERT ON workspaces BEGIN SELECT 1; END;").unwrap(),
                }
            }
        }
        assert!(s.prepare_restore(&path).is_err(), "{damage}");
        assert!(s.restore_backup(true).is_err());
        assert_eq!(before, serde_json::to_value(s.snapshot().unwrap()).unwrap());
    }
}

#[test]
fn interrupted_sqlite_restore_transaction_preserves_destination() {
    let (dir, mut s, _, p) = setup();
    for _ in 0..100 {
        let mut data = input(&p);
        data.description = "x".repeat(50000);
        s.save_issue(None, data).unwrap();
    }
    let path = dir.path().join("large.forma-backup");
    s.export_backup(&path).unwrap();
    s.delete_project(&p).unwrap();
    let before = serde_json::to_value(s.snapshot().unwrap()).unwrap();
    {
        let source = rusqlite::Connection::open(&path).unwrap();
        let backup = rusqlite::backup::Backup::new(&source, &mut s.repo.conn).unwrap();
        assert!(matches!(
            backup.step(1).unwrap(),
            rusqlite::backup::StepResult::More
        ));
        // Drop before completion emulates interruption: SQLite rolls back the destination.
    }
    drop(s);
    let s = Service::open(&dir.path().join("test.sqlite3")).unwrap();
    assert_eq!(before, serde_json::to_value(s.snapshot().unwrap()).unwrap());
    assert_eq!(
        s.repo
            .conn
            .query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "ok"
    );
}

#[test]
#[ignore = "manual representative dataset benchmark; run with --ignored --nocapture"]
fn productivity_benchmark() {
    let (_, mut s, _, p) = setup();
    let start = std::time::Instant::now();
    let tx = s.repo.conn.transaction().unwrap();
    let size: i64 = std::env::var("FORMA_BENCH_ISSUES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10000);
    for n in 1..=size {
        tx.execute("INSERT INTO issues(id,project_id,number,title,description,status,priority,position,due_date,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,'high',?3,'2028-02-29','2026-10-15T00:00:00Z','2026-10-15T00:00:00Z')",
          rusqlite::params![uuid::Uuid::new_v4().to_string(),p,n,format!("Task {n}"),"Representative description ".repeat(20),["backlog","todo","in_progress","done","cancelled"][(n%5) as usize]]).unwrap();
    }
    tx.commit().unwrap();
    println!("seed {size}: {:?}", start.elapsed());
    for (name, search, priority) in [
        ("listing", "", None),
        ("combined search", "Task 99", Some("high".to_string())),
    ] {
        let mut q = query(&p);
        q.search = search.into();
        q.priority = priority;
        let start = std::time::Instant::now();
        for _ in 0..30 {
            assert!(s.issues(q.clone()).unwrap().issues.len() <= 25);
        }
        println!(
            "{name}: {:.3} ms/request (30 samples)",
            start.elapsed().as_secs_f64() * 1000.0 / 30.0
        );
    }
    let start = std::time::Instant::now();
    for _ in 0..10 {
        for status in ["backlog", "todo", "in_progress", "done", "cancelled"] {
            let mut q = query(&p);
            q.status = Some(status.into());
            assert_eq!(s.issues(q).unwrap().issues.len(), 25);
        }
    }
    println!(
        "board 5x25: {:.3} ms/load (10 samples)",
        start.elapsed().as_secs_f64() * 1000.0 / 10.0
    );
    let key = s.issues(query(&p)).unwrap().issues[0].id.clone();
    let start = std::time::Instant::now();
    s.move_issue(&p, &key, "done", None).unwrap();
    println!("move/normalize: {:?}", start.elapsed());
    for row in s.repo.conn.prepare("EXPLAIN QUERY PLAN SELECT id FROM issues WHERE project_id=?1 AND archived=0 AND status='todo' ORDER BY position,number,id LIMIT 25").unwrap().query_map([&p],|r|r.get::<_,String>(3)).unwrap() {
        println!("board query plan: {}",row.unwrap());
    }
}

#[test]
fn relative_anchor_moves_preserve_hidden_cards_and_archive_scope() {
    let (_, mut s, _, p) = setup();
    let cards = (0..5)
        .map(|_| s.save_issue(None, input(&p)).unwrap())
        .collect::<Vec<_>>();
    s.move_issue_relative(&p, &cards[0].id, "todo", Some(&cards[2].id), true)
        .unwrap();
    let ordered = s.issues(query(&p)).unwrap().issues;
    assert_eq!(
        ordered.iter().map(|i| i.id.as_str()).collect::<Vec<_>>(),
        [1, 2, 0, 3, 4].map(|n| cards[n].id.as_str())
    );
    s.archive_issue(&p, &cards[4].id, true).unwrap();
    assert!(s
        .move_issue(&p, &cards[0].id, "todo", Some(&cards[4].id))
        .is_err());
    s.move_issue(&p, &cards[4].id, "done", None).unwrap();
    assert!(s.issue(&p, &cards[4].id).unwrap().archived);
}
#[test]
fn backup_cannot_replace_live_database_or_its_sidecars() {
    let (dir, s, _, _) = setup();
    assert!(s.export_backup(&dir.path().join("test.sqlite3")).is_err());
    assert!(s
        .export_backup(&dir.path().join("test.sqlite3-wal"))
        .is_err());
    assert!(!s.snapshot().unwrap().workspaces.is_empty());
}
