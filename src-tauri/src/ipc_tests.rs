//! Uses Tauri's test transport with the production handler and real SQLite.
//! It does not substitute a fake repository or require a live WebView.
use crate::{commands, service::Service};
use serde_json::{json, Value};
use tauri::{
    test::{mock_builder, mock_context, noop_assets, MockRuntime},
    WebviewWindow,
};
fn invoke(
    w: &WebviewWindow<MockRuntime>,
    cmd: &str,
    body: Value,
) -> std::result::Result<Value, Value> {
    tauri::test::get_ipc_response(
        w,
        tauri::webview::InvokeRequest {
            cmd: cmd.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: if cfg!(windows) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: tauri::ipc::InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.into(),
        },
    )
    .map(|r| r.deserialize().unwrap())
}
#[test]
fn production_ipc_contract_issue_label_subtask_activity_and_restart() {
    let d = tempfile::tempdir().unwrap();
    let path = d.path().join("ipc.sqlite3");
    let app = mock_builder()
        .manage(std::sync::Mutex::new(Service::open(&path).unwrap()))
        .invoke_handler(commands::handler())
        .build(mock_context(noop_assets()))
        .unwrap();
    let w = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let state = invoke(&w, "create_workspace", json!({"name":"IPC smoke"})).unwrap();
    let workspace = state["activeWorkspaceId"].as_str().unwrap();
    let state=invoke(&w,"save_project",json!({"id":null,"input":{"workspaceId":workspace,"name":"Native contract","description":"","color":"#112233","icon":"code","status":"active"}})).unwrap();
    let project = state["projects"][0]["id"].as_str().unwrap();
    let labels = invoke(
        &w,
        "save_label",
        json!({"id":null,"input":{"workspaceId":workspace,"name":"Bug","color":"#ff0000"}}),
    )
    .unwrap();
    let mut input = json!({"projectId":project,"title":"Persist through IPC","description":"**Markdown**","status":"todo","priority":"high","dueDate":"2026-12-01","parentId":null,"labelIds":[labels[0]["id"]],"position":1});
    let issue = invoke(&w, "save_issue", json!({"id":null,"input":input})).unwrap();
    let key = issue["id"].as_str().unwrap();
    assert_eq!(issue["identifier"], "F1-1");
    input["title"] = json!("Child");
    input["parentId"] = json!(key);
    let child = invoke(&w, "save_issue", json!({"id":null,"input":input})).unwrap();
    assert_eq!(
        invoke(
            &w,
            "subtasks",
            json!({"projectId":project,"id":key,"offset":0,"limit":20})
        )
        .unwrap()["issues"][0]["id"],
        child["id"]
    );
    input["parentId"] = Value::Null;
    input["status"] = json!("done");
    input["title"] = json!("Edited through IPC");
    invoke(&w, "save_issue", json!({"id":key,"input":input})).unwrap();
    let page=invoke(&w,"issues",json!({"query":{"projectId":project,"search":"Edited","status":"done","priority":"high","archived":false,"sort":"position","offset":0,"limit":25}})).unwrap();
    assert_eq!(page["total"], 1);
    let activity = invoke(
        &w,
        "issue_activity",
        json!({"projectId":project,"id":key,"offset":0,"limit":20}),
    )
    .unwrap();
    assert_eq!(activity["total"], 2);
    invoke(
        &w,
        "set_project_view",
        json!({"projectId":project,"view":"board"}),
    )
    .unwrap();
    assert_eq!(
        invoke(&w, "project_view", json!({"projectId":project})).unwrap(),
        "board"
    );
    invoke(
        &w,
        "move_issue",
        json!({"projectId":project,"id":child["id"],"status":"done","before":key,"after":true}),
    )
    .unwrap();
    let page=invoke(&w,"issues",json!({"query":{"projectId":project,"search":"","status":"done","priority":"high","labelId":labels[0]["id"],"dueFrom":"2026-12-01","dueTo":"2026-12-01","archived":false,"sort":"position","offset":0,"limit":25}})).unwrap();
    assert_eq!(page["total"], 2);
    assert_eq!(page["issues"][1]["id"], child["id"]);
    assert_eq!(
        invoke(&w, "set_theme", json!({"theme":"dark"})).unwrap()["theme"],
        "dark"
    );
    assert_eq!(
        invoke(&w, "restore_backup", json!({"confirmed":false})).unwrap_err()["code"],
        "validation"
    );

    let error = invoke(
        &w,
        "delete_issue",
        json!({"projectId":uuid::Uuid::new_v4().to_string(),"id":key}),
    )
    .unwrap_err();
    assert_eq!(error["code"], "not_found");
    invoke(
        &w,
        "archive_issue",
        json!({"projectId":project,"id":key,"archived":true}),
    )
    .unwrap();
    invoke(
        &w,
        "archive_issue",
        json!({"projectId":project,"id":key,"archived":false}),
    )
    .unwrap();
    // The native picker itself is exercised by the opt-in graphical smoke. Seed
    // its Rust-only result here, then exercise every read/link through IPC.
    let root = d.path().join("repo");
    let git = git2::Repository::init(&root).unwrap();
    let tree_id = git.index().unwrap().write_tree().unwrap();
    let tree = git.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("IPC author", "ipc@example.test").unwrap();
    let oid = git
        .commit(Some("HEAD"), &sig, &sig, "Fix F1-1", &tree, &[])
        .unwrap();
    git.branch("feature/F1-1", &git.find_commit(oid).unwrap(), false)
        .unwrap();
    use tauri::Manager;
    let repository = app
        .state::<std::sync::Mutex<Service>>()
        .lock()
        .unwrap()
        .git_connect(project, &root)
        .unwrap();
    assert_eq!(
        invoke(&w, "git_connection", json!({"projectId":project})).unwrap()["id"],
        repository.id
    );
    assert_eq!(
        invoke(
            &w,
            "git_overview",
            json!({"projectId":project,"repositoryId":repository.id,"offset":0,"limit":20})
        )
        .unwrap()["commits"][0]["hash"],
        oid.to_string()
    );
    assert_eq!(invoke(&w,"git_branches",json!({"projectId":project,"repositoryId":repository.id,"search":"F1-1","offset":0,"limit":25})).unwrap()["branches"][0]["name"],"feature/F1-1");
    let mut link = json!({"projectId":project,"issueId":key,"repositoryId":repository.id,"kind":"branch","target":"feature/F1-1","confirmed":false});
    assert_eq!(
        invoke(&w, "git_link", json!({"input":link})).unwrap_err()["code"],
        "validation"
    );
    link["confirmed"] = json!(true);
    let links = invoke(&w, "git_link", json!({"input":link})).unwrap();
    assert_eq!(links[0]["target"], "feature/F1-1");
    assert_eq!(
        invoke(&w, "git_links", json!({"projectId":project,"issueId":key}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    // Native executable picker is represented by its Rust-only authorization
    // result. Every preview/run/history/review action uses real production IPC.
    app.state::<std::sync::Mutex<Service>>()
        .lock()
        .unwrap()
        .agent_authorize("codex", &crate::agent_tests::fixture_executable())
        .unwrap();
    assert!(invoke(&w, "agent_onboarding", json!({"provider":"untrusted"})).is_err());
    assert!(invoke(
        &w,
        "agent_authorize_discovered",
        json!({"token":"/arbitrary/executable","confirmed":true})
    )
    .is_err());
    assert!(invoke(
        &w,
        "agent_authorize_discovered",
        json!({"token":"fake","confirmed":false})
    )
    .is_err());
    assert!(invoke(
        &w,
        "agent_login",
        json!({"provider":"codex","approved":false})
    )
    .is_err());
    let setup = invoke(&w, "agent_onboarding", json!({"provider":"codex"})).unwrap();
    assert_eq!(setup["authentication"], "authenticated");
    assert_eq!(setup["ready"], true);
    assert_eq!(invoke(&w, "agent_login_status", json!({})).unwrap(), "idle");
    invoke(&w, "agent_cancel_login", json!({})).unwrap();
    let providers = invoke(&w, "agent_providers", json!({})).unwrap();
    assert!(providers
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["settings"]["provider"] == "codex" && p["available"] == true));
    invoke(&w, "agent_save_preferences", json!({"preferred":"codex"})).unwrap();
    assert_eq!(invoke(&w, "agent_preferred", json!({})).unwrap(), "codex");
    let preview = invoke(
        &w,
        "agent_preview",
        json!({"projectId":project,"issueId":key,"provider":"codex"}),
    )
    .unwrap();
    assert!(invoke(
        &w,
        "agent_start",
        json!({"token":preview["token"],"approved":false,"elevatedApproved":false})
    )
    .is_err());
    let run = invoke(
        &w,
        "agent_start",
        json!({"token":preview["token"],"approved":true,"elevatedApproved":false}),
    )
    .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let runs = invoke(
            &w,
            "agent_runs",
            json!({"projectId":project,"issueId":key,"offset":0}),
        )
        .unwrap();
        if runs[0]["status"] == "Completed" {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "IPC run did not complete"
        );
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    assert!(
        invoke(&w, "agent_review", json!({"runId":run["id"]})).unwrap()["additions"]
            .as_u64()
            .unwrap()
            > 0
    );
    invoke(
        &w,
        "agent_mark_reviewed",
        json!({"runId":run["id"],"decision":"accepted"}),
    )
    .unwrap();
    assert_eq!(
        invoke(&w, "agent_recent", json!({"offset":0})).unwrap()[0]["review"],
        "accepted"
    );
    invoke(&w, "agent_cancel", json!({"runId":run["id"]})).unwrap();
    assert_eq!(invoke(&w, "agent_concurrency", json!({})).unwrap(), 1);
    invoke(&w, "agent_set_concurrency", json!({"maximum":2})).unwrap();
    assert!(invoke(&w, "agent_set_concurrency", json!({"maximum":3})).is_err());
    let queued_preview = invoke(
        &w,
        "agent_preview",
        json!({"projectId":project,"issueId":key,"provider":"codex"}),
    )
    .unwrap();
    assert!(invoke(
        &w,
        "agent_enqueue",
        json!({"token":queued_preview["token"],"approved":false,"elevatedApproved":false})
    )
    .is_err());
    let task = invoke(
        &w,
        "agent_enqueue",
        json!({"token":queued_preview["token"],"approved":true,"elevatedApproved":false}),
    )
    .unwrap();
    assert_eq!(
        invoke(&w, "agent_queue", json!({})).unwrap()[0]["status"],
        "Approved"
    );
    invoke(&w, "agent_queue_move", json!({"taskId":task,"before":null})).unwrap();
    invoke(&w, "agent_queue_cancel", json!({"taskId":task})).unwrap();
    assert!(invoke(&w, "agent_start_queue", json!({"all":false}))
        .unwrap()
        .as_array()
        .unwrap()
        .is_empty());
    invoke(&w, "agent_queue_stop", json!({})).unwrap();
    assert_eq!(
        invoke(&w, "agent_worktrees", json!({"offset":0})).unwrap()[0]["run"]["id"],
        run["id"]
    );
    assert!(invoke(
        &w,
        "agent_cleanup",
        json!({"runId":run["id"],"confirmed":false})
    )
    .is_err());
    assert_eq!(
        invoke(&w, "test_project", json!({"runId":run["id"]})).unwrap(),
        project
    );
    {
        use tauri::Manager;
        app.state::<std::sync::Mutex<Service>>()
            .lock()
            .unwrap()
            .test_authorize(project, &crate::agent_tests::fixture_executable())
            .unwrap();
    }
    let mut test_cmd = invoke(&w, "test_command", json!({"projectId":project})).unwrap();
    test_cmd["arguments"] = json!(["--forma-test"]);
    invoke(&w, "test_save_command", json!({"input":test_cmd})).unwrap();
    let tp = invoke(&w, "test_preview", json!({"runId":run["id"]})).unwrap();
    assert!(invoke(
        &w,
        "test_start",
        json!({"token":tp["token"],"approved":false})
    )
    .is_err());
    let tid = invoke(
        &w,
        "test_start",
        json!({"token":tp["token"],"approved":true}),
    )
    .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let tests = invoke(&w, "test_runs", json!({"runId":run["id"]})).unwrap();
        if tests[0]["status"] == "Completed" {
            assert_eq!(tests[0]["exitCode"], 0);
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    invoke(&w, "test_cancel", json!({"testId":tid})).unwrap();

    // Exercise the native dispatcher command, including its independently owned
    // app handle, rather than only calling the service scheduler directly.
    let queued_preview = invoke(
        &w,
        "agent_preview",
        json!({"projectId":project,"issueId":key,"provider":"codex"}),
    )
    .unwrap();
    invoke(
        &w,
        "agent_enqueue",
        json!({"token":queued_preview["token"],"approved":true,"elevatedApproved":false}),
    )
    .unwrap();
    let dispatched = invoke(&w, "agent_start_queue", json!({"all":true})).unwrap();
    assert_eq!(dispatched.as_array().unwrap().len(), 1);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        let recent = invoke(&w, "agent_recent", json!({"offset":0})).unwrap();
        let dispatched_run = recent
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == dispatched[0]["id"])
            .unwrap();
        if dispatched_run["status"] == "Completed" {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    invoke(&w, "agent_queue_stop", json!({})).unwrap();

    let check = Service::open(&path).unwrap();
    assert_eq!(check.git_links(project, key).unwrap().len(), 1);
    invoke(&w, "git_disconnect", json!({"projectId":project})).unwrap();
    assert!(invoke(&w, "git_connection", json!({"projectId":project}))
        .unwrap()
        .is_null());
    assert_eq!(
        invoke(&w, "git_links", json!({"projectId":project,"issueId":key})).unwrap()[0]
            ["connected"],
        false
    );
    invoke(
        &w,
        "git_unlink",
        json!({"projectId":project,"issueId":key,"linkId":links[0]["id"]}),
    )
    .unwrap();
    // Reopen a separate connection after all IPC writes have committed.
    let reopened = Service::open(&path).unwrap();
    let saved = reopened.issue(project, key).unwrap();
    assert_eq!(saved.title, "Edited through IPC");
    assert_eq!(saved.label_ids.len(), 1);
    assert_eq!(reopened.subtasks(project, key, 0, 20).unwrap().total, 1);
    invoke(
        &w,
        "delete_label",
        json!({"workspaceId":workspace,"id":labels[0]["id"]}),
    )
    .unwrap();
    invoke(
        &w,
        "delete_issue",
        json!({"projectId":project,"id":child["id"]}),
    )
    .unwrap();
}
