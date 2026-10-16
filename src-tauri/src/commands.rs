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
        agent_worktrees,
        agent_cleanup,
        test_command,
        test_authorize,
        test_save_command,
        test_preview,
        test_start,
        test_runs,
        test_cancel,
        agent_concurrency,
        agent_set_concurrency,
        agent_enqueue,
        agent_queue,
        agent_queue_cancel,
        agent_queue_move,
        agent_start_queue,
        agent_queue_stop,
        test_project,
        agent_recent,
        agent_providers,
        agent_onboarding,
        agent_authorize_discovered,
        agent_login,
        agent_cancel_login,
        agent_login_status,
        agent_setup_docs,
        agent_preferred,
        agent_save_preferences,
        agent_save_settings,
        agent_authorize,
        agent_preview,
        agent_start,
        agent_runs,
        agent_cancel,
        agent_review,
        agent_mark_reviewed,
        agent_open_worktree,
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

#[tauri::command]
pub async fn agent_providers<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<crate::agent_types::ProviderStatus>> {
    git_task(app, |s| s.agent_providers()).await
}
#[tauri::command]
pub async fn agent_preferred<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<String> {
    git_task(app, |s| s.agent_preferred()).await
}
#[tauri::command]
pub async fn agent_save_preferences<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    preferred: String,
) -> Result<()> {
    git_task(app, move |s| s.agent_save_preferences(&preferred)).await
}
#[tauri::command]
pub async fn agent_save_settings<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: crate::agent_types::AgentSettings,
) -> Result<()> {
    git_task(app, move |s| s.agent_save_settings(input)).await
}
#[tauri::command]
pub async fn agent_authorize<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    provider: String,
) -> Result<bool> {
    crate::agent_provider::provider(&provider)?;
    let file = rfd::AsyncFileDialog::new()
        .set_title("Authorize agent executable — version/help will be executed")
        .pick_file()
        .await;
    match file {
        Some(file) => {
            git_task(app, move |s| {
                s.agent_authorize(&provider, file.path())?;
                Ok(true)
            })
            .await
        }
        None => Ok(false),
    }
}
#[tauri::command]
pub async fn agent_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
    issue_id: String,
    provider: String,
) -> Result<crate::agent_types::AgentPreview> {
    git_task(app, move |s| {
        s.agent_preview(&project_id, &issue_id, &provider)
    })
    .await
}
#[tauri::command]
pub async fn agent_start<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    token: String,
    approved: bool,
    elevated_approved: bool,
) -> Result<crate::agent_types::AgentRun> {
    git_task(app, move |s| {
        s.agent_start(&token, approved, elevated_approved)
    })
    .await
}
#[tauri::command]
pub async fn agent_runs<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
    issue_id: String,
    offset: usize,
) -> Result<Vec<crate::agent_types::AgentRun>> {
    git_task(app, move |s| s.agent_runs(&project_id, &issue_id, offset)).await
}
#[tauri::command]
pub async fn agent_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    run_id: String,
) -> Result<()> {
    git_task(app, move |s| s.agent_cancel(&run_id)).await
}
#[tauri::command]
pub async fn agent_review<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    run_id: String,
) -> Result<crate::agent_types::AgentReview> {
    git_task(app, move |s| s.agent_review(&run_id)).await
}
#[tauri::command]
pub async fn agent_mark_reviewed<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    run_id: String,
    decision: String,
) -> Result<()> {
    git_task(app, move |s| s.agent_mark_reviewed(&run_id, &decision)).await
}
#[tauri::command]
pub async fn agent_open_worktree<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    run_id: String,
) -> Result<()> {
    git_task(app, move |s| s.agent_open_worktree(&run_id)).await
}

#[tauri::command]
pub async fn agent_recent<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    offset: usize,
) -> Result<Vec<crate::agent_types::AgentRun>> {
    git_task(app, move |s| s.agent_recent(offset)).await
}

#[tauri::command]
pub async fn agent_concurrency<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<usize> {
    git_task(app, |s| s.agent_concurrency()).await
}
#[tauri::command]
pub async fn agent_set_concurrency<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    maximum: usize,
) -> Result<()> {
    git_task(app, move |s| s.agent_set_concurrency(maximum)).await
}
#[tauri::command]
pub async fn agent_enqueue<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    token: String,
    approved: bool,
    elevated_approved: bool,
) -> Result<String> {
    git_task(app, move |s| {
        s.agent_enqueue(&token, approved, elevated_approved)
    })
    .await
}
#[tauri::command]
pub async fn agent_queue<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
) -> Result<Vec<crate::agent_types::QueuedTask>> {
    git_task(app, |s| s.agent_queue()).await
}
#[tauri::command]
pub async fn agent_queue_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    task_id: String,
) -> Result<()> {
    git_task(app, move |s| s.agent_queue_cancel(&task_id)).await
}
#[tauri::command]
pub async fn agent_queue_move<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    task_id: String,
    before: Option<String>,
) -> Result<()> {
    git_task(app, move |s| {
        s.agent_queue_move(&task_id, before.as_deref())
    })
    .await
}
#[tauri::command]
pub async fn agent_start_queue<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    all: bool,
) -> Result<Vec<crate::agent_types::AgentRun>> {
    if !all {
        return git_task(app, move |s| s.agent_start_queue(false)).await;
    }
    let (runs, generation) = git_task(app.clone(), |s| {
        let runs = s.agent_queue_begin()?;
        Ok((runs, s.agents.queue_generation))
    })
    .await?;
    // Session state lives only in memory. No queue dispatcher is created at startup.
    let worker_app = app.clone();
    let dispatcher = std::thread::Builder::new()
        .name("approved-queue".into())
        .spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_millis(250));
            use tauri::Manager;
            let state = worker_app.state::<Mutex<Service>>();
            let Ok(mut service) = state.lock() else {
                break;
            };
            if service.agents.queue_generation != generation {
                break;
            }
            match service.agent_queue_tick() {
                Ok(true) => {}
                _ => {
                    service.agent_queue_stop();
                    break;
                }
            }
        });
    if dispatcher.is_err() {
        git_task(app, move |service| {
            if service.agents.queue_generation == generation {
                service.agent_queue_stop();
            }
            Ok(())
        })
        .await?;
        return Err(AppError {
            code: "agent",
            message: "Could not start queue dispatcher. Already launched runs continue; retry Start for remaining tasks.".into(),
        });
    }
    Ok(runs)
}

#[tauri::command]
pub async fn agent_worktrees<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    offset: usize,
) -> Result<Vec<crate::agent_types::WorktreeInfo>> {
    git_task(app, move |s| s.agent_worktrees(offset)).await
}
#[tauri::command]
pub async fn agent_cleanup<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    run_id: String,
    confirmed: bool,
) -> Result<()> {
    git_task(app, move |s| s.agent_cleanup(&run_id, confirmed)).await
}
#[tauri::command]
pub async fn test_command<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
) -> Result<Option<crate::agent_types::TestCommand>> {
    git_task(app, move |s| s.test_command(&project_id)).await
}
#[tauri::command]
pub async fn test_authorize<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    project_id: String,
) -> Result<bool> {
    let file = rfd::AsyncFileDialog::new()
        .set_title("Authorize independent test runner executable")
        .pick_file()
        .await;
    match file {
        Some(file) => {
            git_task(app, move |s| {
                s.test_authorize(&project_id, file.path())?;
                Ok(true)
            })
            .await
        }
        None => Ok(false),
    }
}
#[tauri::command]
pub async fn test_save_command<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    input: crate::agent_types::TestCommand,
) -> Result<()> {
    git_task(app, move |s| s.test_save_command(input)).await
}
#[tauri::command]
pub async fn test_preview<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    run_id: String,
) -> Result<crate::agent_types::TestPreview> {
    git_task(app, move |s| s.test_preview(&run_id)).await
}
#[tauri::command]
pub async fn test_start<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    token: String,
    approved: bool,
) -> Result<String> {
    git_task(app, move |s| s.test_start(&token, approved)).await
}
#[tauri::command]
pub async fn test_runs<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    run_id: String,
) -> Result<Vec<crate::agent_types::IndependentTest>> {
    git_task(app, move |s| s.test_runs(&run_id)).await
}
#[tauri::command]
pub async fn test_cancel<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    test_id: String,
) -> Result<()> {
    git_task(app, move |s| s.test_cancel(&test_id)).await
}

#[tauri::command]
pub async fn agent_queue_stop<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
    git_task(app, |s| {
        s.agent_queue_stop();
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn test_project<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    run_id: String,
) -> Result<String> {
    git_task(app, move |s| s.test_project(&run_id)).await
}

#[tauri::command]
pub async fn agent_onboarding<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    provider: String,
) -> Result<crate::onboarding::Onboarding> {
    git_task(app, move |s| s.agent_onboarding(&provider)).await
}
#[tauri::command]
pub async fn agent_authorize_discovered<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    token: String,
    confirmed: bool,
) -> Result<()> {
    git_task(app, move |s| {
        s.agent_authorize_discovered(&token, confirmed)
    })
    .await
}
#[tauri::command]
pub async fn agent_login<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    provider: String,
    approved: bool,
) -> Result<()> {
    git_task(app, move |s| s.agent_login(&provider, approved)).await
}
#[tauri::command]
pub async fn agent_cancel_login<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<()> {
    git_task(app, |s| s.agent_cancel_login()).await
}
#[tauri::command]
pub async fn agent_setup_docs(provider: String) -> Result<()> {
    crate::agent_provider::provider(&provider)?;
    let url = if provider == "codex" {
        "https://developers.openai.com/codex/cli/"
    } else {
        "https://code.claude.com/docs/en/quickstart"
    };
    opener::open(url).map_err(|_| {
        crate::agent_provider::error(
            "Could not open the official installation guide. Copy its address into your browser.",
        )
    })
}

#[tauri::command]
pub async fn agent_login_status<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> Result<String> {
    git_task(app, |s| {
        Ok(s.agents
            .login
            .as_ref()
            .map(|l| l.state.lock().unwrap_or_else(|e| e.into_inner()).clone())
            .unwrap_or_else(|| "idle".into()))
    })
    .await
}
