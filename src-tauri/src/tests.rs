use crate::{domain::*, service::Service};
fn service() -> (tempfile::TempDir, Service) {
    let dir = tempfile::tempdir().unwrap();
    let service = Service::open(&dir.path().join("test.sqlite3")).unwrap();
    (dir, service)
}
fn input(workspace_id: &str) -> ProjectInput {
    ProjectInput {
        workspace_id: workspace_id.into(),
        name: "Engine".into(),
        description: "Ship it".into(),
        icon: "code".into(),
        color: "#8b5cf6".into(),
        status: "active".into(),
    }
}
#[test]
fn workspace_crud_selection_and_fallback() {
    let (_dir, mut s) = service();
    assert!(s.snapshot().unwrap().workspaces.is_empty());
    let first = s
        .create_workspace("  Studio  ")
        .unwrap()
        .active_workspace_id
        .unwrap();
    let second = s
        .create_workspace("Personal")
        .unwrap()
        .active_workspace_id
        .unwrap();
    assert_eq!(
        s.rename_workspace(&first, "Renamed").unwrap().workspaces[0].name,
        "Renamed"
    );
    assert_eq!(
        s.select_workspace(&first).unwrap().active_workspace_id,
        Some(first.clone())
    );
    let snapshot = s.delete_workspace(&first).unwrap();
    assert_eq!(snapshot.workspaces.len(), 1);
    assert_eq!(snapshot.active_workspace_id, Some(second.clone()));
    assert!(s.delete_workspace(&first).is_err());
    let snapshot = s.delete_workspace(&second).unwrap();
    assert!(snapshot.workspaces.is_empty());
    assert_eq!(snapshot.active_workspace_id, None);
}
#[test]
fn project_crud_archive_and_workspace_isolation() {
    let (_dir, mut s) = service();
    let first = s
        .create_workspace("A")
        .unwrap()
        .active_workspace_id
        .unwrap();
    let created = s
        .save_project(None, input(&first))
        .unwrap()
        .projects
        .remove(0);
    assert_eq!(created.name, "Engine");
    let mut edit = input(&first);
    edit.name = "Engine v2".into();
    edit.description = "Updated".into();
    edit.icon = "rocket".into();
    edit.color = "#10b981".into();
    edit.status = "completed".into();
    let project = s
        .save_project(Some(&created.id), edit)
        .unwrap()
        .projects
        .remove(0);
    assert_eq!(project.name, "Engine v2");
    assert_eq!(project.description, "Updated");
    assert_eq!(project.icon, "rocket");
    assert_eq!(project.color, "#10b981");
    assert_eq!(project.status, "completed");
    assert!(s.archive_project(&project.id, true).unwrap().projects[0].archived);
    assert!(!s.archive_project(&project.id, false).unwrap().projects[0].archived);
    let second = s
        .create_workspace("B")
        .unwrap()
        .active_workspace_id
        .unwrap();
    assert!(s.snapshot().unwrap().projects.is_empty());
    assert!(s.save_project(Some(&project.id), input(&second)).is_err());
    assert_eq!(s.select_workspace(&first).unwrap().projects.len(), 1);
    assert!(s.delete_project(&project.id).unwrap().projects.is_empty());
    assert!(s.delete_project(&project.id).is_err());
}
#[test]
fn database_and_selection_survive_reopen() {
    let (dir, mut s) = service();
    let first = s
        .create_workspace("Persist")
        .unwrap()
        .active_workspace_id
        .unwrap();
    s.save_project(None, input(&first)).unwrap();
    s.create_workspace("Other").unwrap();
    s.select_workspace(&first).unwrap();
    drop(s);
    let s = Service::open(&dir.path().join("test.sqlite3")).unwrap();
    let snapshot = s.snapshot().unwrap();
    assert_eq!(snapshot.active_workspace_id, Some(first));
    assert_eq!(snapshot.workspaces.len(), 2);
    assert_eq!(snapshot.projects[0].description, "Ship it");
}
#[test]
fn migration_is_versioned_idempotent_and_enforces_foreign_keys() {
    let (dir, mut s) = service();
    let conn = &s.repo.conn;
    assert_eq!(
        conn.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        3
    );
    assert_eq!(
        conn.pragma_query_value(None, "foreign_keys", |r| r.get::<_, i64>(0))
            .unwrap(),
        1
    );
    for table in [
        "workspaces",
        "projects",
        "issues",
        "labels",
        "issue_labels",
        "activity_events",
        "app_state",
    ] {
        assert!(conn.prepare(&format!("SELECT * FROM {table}")).is_ok());
    }
    assert!(conn
        .execute(
            "INSERT INTO issues(id,project_id,title,description,status,created_at,updated_at,number) VALUES('bad','missing','test','','backlog','now','now',1)",
            []
        )
        .is_err());
    let id = s
        .create_workspace("A")
        .unwrap()
        .active_workspace_id
        .unwrap();
    let p = s.save_project(None, input(&id)).unwrap().projects[0]
        .id
        .clone();
    s.repo
        .conn
        .execute(
            "INSERT INTO issues(id,project_id,title,description,status,created_at,updated_at,number) VALUES('issue',?1,'test','','backlog','now','now',1)",
            [p],
        )
        .unwrap();
    s.repo
        .conn
        .execute(
            "INSERT INTO labels VALUES('label',?1,'bug','#ff0000','now','now')",
            [&id],
        )
        .unwrap();
    s.repo
        .conn
        .execute("INSERT INTO issue_labels VALUES('issue','label','now')", [])
        .unwrap();
    s.delete_workspace(&id).unwrap();
    for table in [
        "projects",
        "issues",
        "labels",
        "issue_labels",
        "activity_events",
    ] {
        assert_eq!(
            s.repo
                .conn
                .query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r
                    .get::<_, i64>(0))
                .unwrap(),
            0
        );
    }
    drop(s);
    let s = Service::open(&dir.path().join("test.sqlite3")).unwrap();
    assert!(s.snapshot().unwrap().workspaces.is_empty());
}
#[test]
fn validation_rejects_invalid_data_without_writes() {
    let (_dir, mut s) = service();
    for name in ["", "   ", "bad\nname", &"x".repeat(81)] {
        assert_eq!(s.create_workspace(name).unwrap_err().code, "validation");
    }
    let id = s
        .create_workspace("A")
        .unwrap()
        .active_workspace_id
        .unwrap();
    for field in [
        "name",
        "description",
        "icon",
        "color",
        "status",
        "workspace",
    ] {
        let mut p = input(&id);
        match field {
            "name" => p.name = " ".into(),
            "description" => p.description = "x".repeat(4001),
            "icon" => p.icon = "bad".into(),
            "color" => p.color = "red".into(),
            "status" => p.status = "bad".into(),
            _ => p.workspace_id = "bad".into(),
        }
        assert_eq!(s.save_project(None, p).unwrap_err().code, "validation");
    }
    assert!(s.snapshot().unwrap().projects.is_empty());
    assert!(s.select_workspace("invalid").is_err());
    assert!(s
        .select_workspace(&uuid::Uuid::new_v4().to_string())
        .is_err());
}
#[test]
fn newer_database_is_not_downgraded() {
    let (dir, s) = service();
    s.repo.conn.pragma_update(None, "user_version", 99).unwrap();
    drop(s);
    assert!(Service::open(&dir.path().join("test.sqlite3")).is_err());
}
#[test]
fn activity_changes_are_transactional() {
    let (_dir, mut s) = service();
    let id = s
        .create_workspace("A")
        .unwrap()
        .active_workspace_id
        .unwrap();
    s.repo.conn.execute_batch("CREATE TRIGGER reject_activity BEFORE INSERT ON activity_events BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(s.save_project(None, input(&id)).is_err());
    assert!(s.snapshot().unwrap().projects.is_empty());
    assert!(s.rename_workspace(&id, "Changed").is_err());
    assert_eq!(s.snapshot().unwrap().workspaces[0].name, "A");
}
