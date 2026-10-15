use crate::{domain::*, service::Service};
use std::sync::Mutex;
type State<'a> = tauri::State<'a, Mutex<Service>>;
fn locked<'a>(state: &'a State<'_>) -> Result<std::sync::MutexGuard<'a, Service>> {
    state.lock().map_err(|_| AppError {
        code: "internal",
        message: "Local data is temporarily unavailable. Restart Forma.".into(),
    })
}
#[tauri::command]
pub fn snapshot(state: State<'_>) -> Result<Snapshot> {
    locked(&state)?.snapshot()
}
#[tauri::command]
pub fn create_workspace(state: State<'_>, name: String) -> Result<Snapshot> {
    locked(&state)?.create_workspace(&name)
}
#[tauri::command]
pub fn rename_workspace(state: State<'_>, id: String, name: String) -> Result<Snapshot> {
    locked(&state)?.rename_workspace(&id, &name)
}
#[tauri::command]
pub fn select_workspace(state: State<'_>, id: String) -> Result<Snapshot> {
    locked(&state)?.select_workspace(&id)
}
#[tauri::command]
pub fn delete_workspace(state: State<'_>, id: String) -> Result<Snapshot> {
    locked(&state)?.delete_workspace(&id)
}
#[tauri::command]
pub fn save_project(state: State<'_>, id: Option<String>, input: ProjectInput) -> Result<Snapshot> {
    locked(&state)?.save_project(id.as_deref(), input)
}
#[tauri::command]
pub fn archive_project(state: State<'_>, id: String, archived: bool) -> Result<Snapshot> {
    locked(&state)?.archive_project(&id, archived)
}
#[tauri::command]
pub fn delete_project(state: State<'_>, id: String) -> Result<Snapshot> {
    locked(&state)?.delete_project(&id)
}

#[tauri::command]
pub async fn issues(state: State<'_>, query: IssueQuery) -> Result<IssuePage> {
    locked(&state)?.issues(query)
}

#[tauri::command]
pub fn issue(state: State<'_>, project_id: String, id: String) -> Result<Issue> {
    locked(&state)?.issue(&project_id, &id)
}

#[tauri::command]
pub fn save_issue(state: State<'_>, id: Option<String>, input: IssueInput) -> Result<Issue> {
    locked(&state)?.save_issue(id.as_deref(), input)
}

#[tauri::command]
pub fn archive_issue(
    state: State<'_>,
    project_id: String,
    id: String,
    archived: bool,
) -> Result<()> {
    locked(&state)?.archive_issue(&project_id, &id, archived)
}

#[tauri::command]
pub fn delete_issue(state: State<'_>, project_id: String, id: String) -> Result<()> {
    locked(&state)?.delete_issue(&project_id, &id)
}

#[tauri::command]
pub fn labels(state: State<'_>, workspace_id: String) -> Result<Vec<Label>> {
    locked(&state)?.labels(&workspace_id)
}

#[tauri::command]
pub fn save_label(state: State<'_>, id: Option<String>, input: LabelInput) -> Result<Vec<Label>> {
    locked(&state)?.save_label(id.as_deref(), input)
}

#[tauri::command]
pub fn delete_label(state: State<'_>, workspace_id: String, id: String) -> Result<Vec<Label>> {
    locked(&state)?.delete_label(&workspace_id, &id)
}

#[tauri::command]
pub fn issue_activity(
    state: State<'_>,
    project_id: String,
    id: String,
    offset: i64,
    limit: i64,
) -> Result<ActivityPage> {
    locked(&state)?.issue_activity(&project_id, &id, offset, limit)
}

#[tauri::command]
pub fn subtasks(
    state: State<'_>,
    project_id: String,
    id: String,
    offset: i64,
    limit: i64,
) -> Result<IssuePage> {
    locked(&state)?.subtasks(&project_id, &id, offset, limit)
}

#[tauri::command]
pub async fn move_issue(
    state: State<'_>,
    project_id: String,
    id: String,
    status: String,
    before: Option<String>,
    after: Option<bool>,
) -> Result<()> {
    locked(&state)?.move_issue_relative(
        &project_id,
        &id,
        &status,
        before.as_deref(),
        after.unwrap_or(false),
    )
}
#[tauri::command]
pub fn project_view(state: State<'_>, project_id: String) -> Result<String> {
    locked(&state)?.project_view(&project_id)
}
#[tauri::command]
pub fn set_project_view(state: State<'_>, project_id: String, view: String) -> Result<()> {
    locked(&state)?.set_project_view(&project_id, &view)
}
pub fn handler<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool + Send + Sync + 'static
{
    tauri::generate_handler![
        git_connection,
        git_connect,
        git_disconnect,
        git_overview,
        git_branches,
        git_links,
        git_link,
        git_unlink,
        set_theme,
        export_backup,
        prepare_restore,
        restore_backup,
        cancel_restore,
        move_issue,
        project_view,
        set_project_view,
        snapshot,
        create_workspace,
        rename_workspace,
        delete_workspace,
        select_workspace,
        save_project,
        archive_project,
        delete_project,
        issues,
        issue,
        save_issue,
        archive_issue,
        delete_issue,
        labels,
        save_label,
        delete_label,
        issue_activity,
        subtasks
    ]
}

#[tauri::command]
pub async fn export_backup(state: State<'_>) -> Result<bool> {
    let file = rfd::AsyncFileDialog::new()
        .set_title("Export Forma backup")
        .add_filter("Forma backup", &["forma-backup"])
        .set_file_name("forma.forma-backup")
        .save_file()
        .await;
    if let Some(file) = file {
        locked(&state)?.export_backup(file.path())?;
        Ok(true)
    } else {
        Ok(false)
    }
}
#[tauri::command]
pub async fn prepare_restore(state: State<'_>) -> Result<bool> {
    let file = rfd::AsyncFileDialog::new()
        .set_title("Select Forma backup")
        .add_filter("Forma backup", &["forma-backup"])
        .pick_file()
        .await;
    if let Some(file) = file {
        locked(&state)?.prepare_restore(file.path())?;
        Ok(true)
    } else {
        locked(&state)?.cancel_restore();
        Ok(false)
    }
}
#[tauri::command]
pub async fn restore_backup(state: State<'_>, confirmed: bool) -> Result<String> {
    locked(&state)?.restore_backup(confirmed)
}
#[tauri::command]
pub fn cancel_restore(state: State<'_>) -> Result<()> {
    locked(&state)?.cancel_restore();
    Ok(())
}

#[tauri::command]
pub fn set_theme(state: State<'_>, theme: String) -> Result<Snapshot> {
    locked(&state)?.set_theme(&theme)
}

// Blocking workers own an app handle; no mutex guard crosses an await.
async fn git_task<R: tauri::Runtime, T: Send + 'static>(
    app: tauri::AppHandle<R>,
    work: impl FnOnce(&mut Service) -> Result<T> + Send + 'static,
) -> Result<T> {
    use tauri::Manager;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<Mutex<Service>>();
        let mut service = locked(&state)?;
        work(&mut service)
    })
    .await
    .map_err(|_| AppError {
        code: "internal",
        message: "Git operation could not finish. Please retry.".into(),
    })?
}
#[tauri::command]
pub async fn git_connection<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
) -> Result<Option<GitRepository>> {
    git_task(app, move |s| s.git_connection(&project_id)).await
}
#[tauri::command]
pub async fn git_connect<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
) -> Result<Option<GitRepository>> {
    let folder = rfd::AsyncFileDialog::new()
        .set_title("Connect Git repository")
        .pick_folder()
        .await;
    match folder {
        Some(folder) => {
            git_task(app, move |s| {
                s.git_connect(&project_id, folder.path()).map(Some)
            })
            .await
        }
        None => Ok(None),
    }
}
#[tauri::command]
pub async fn git_disconnect<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
) -> Result<()> {
    git_task(app, move |s| s.git_disconnect(&project_id)).await
}
#[tauri::command]
pub async fn git_overview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
    repository_id: String,
    offset: usize,
    limit: usize,
) -> Result<GitOverview> {
    git_task(app, move |s| {
        s.git_overview(&project_id, &repository_id, offset, limit)
    })
    .await
}
#[tauri::command]
pub async fn git_branches<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
    repository_id: String,
    search: String,
    offset: usize,
    limit: usize,
) -> Result<GitBranchPage> {
    git_task(app, move |s| {
        s.git_branches(&project_id, &repository_id, &search, offset, limit)
    })
    .await
}
#[tauri::command]
pub async fn git_links<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
    issue_id: String,
) -> Result<Vec<GitLink>> {
    git_task(app, move |s| s.git_links(&project_id, &issue_id)).await
}
#[tauri::command]
pub async fn git_link<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: GitLinkInput,
) -> Result<Vec<GitLink>> {
    git_task(app, move |s| s.git_link(input)).await
}
#[tauri::command]
pub async fn git_unlink<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
    issue_id: String,
    link_id: String,
) -> Result<Vec<GitLink>> {
    git_task(app, move |s| s.git_unlink(&project_id, &issue_id, &link_id)).await
}
