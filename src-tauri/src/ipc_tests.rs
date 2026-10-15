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
