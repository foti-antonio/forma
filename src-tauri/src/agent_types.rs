use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AgentSettings {
    pub provider: String,
    pub executable: Option<String>,
    pub trusted: bool,
    pub max_runtime: u64,
    pub log_limit: usize,
    pub policy: String,
    #[serde(default)]
    pub require_os_isolation: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatus {
    pub settings: AgentSettings,
    pub version: Option<String>,
    pub available: bool,
    pub message: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRun {
    pub issue_identifier: String,
    pub issue_title: String,
    pub id: String,
    pub issue_id: String,
    pub repository_id: String,
    pub provider: String,
    pub worktree: String,
    pub branch: String,
    pub base_commit: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub exit_code: Option<i32>,
    pub result: String,
    pub log: String,
    pub review: Option<String>,
    pub trusted: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPreview {
    pub token: String,
    pub prompt: String,
    pub provider: String,
    pub executable: String,
    pub version: String,
    pub repository: String,
    pub base_commit: String,
    pub policy: String,
    pub max_runtime: u64,
    pub log_limit: usize,
    pub warning: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentReview {
    pub files: Vec<String>,
    pub diff: String,
    pub additions: usize,
    pub deletions: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedTask {
    pub id: String,
    pub issue_id: String,
    pub title: String,
    pub provider: String,
    pub approved_at: String,
    pub expires_at: String,
    pub position: i64,
    pub status: String,
    pub message: String,
    pub run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestCommand {
    pub project_id: String,
    pub executable: String,
    pub trusted: bool,
    pub arguments: Vec<String>,
    pub max_runtime: u64,
    pub log_limit: usize,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestPreview {
    pub token: String,
    pub run_id: String,
    pub command: TestCommand,
    pub working_directory: String,
    pub expires_at: i64,
    #[serde(skip)]
    pub(crate) digest: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndependentTest {
    pub id: String,
    pub run_id: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub working_directory: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub exit_code: Option<i32>,
    pub result: String,
    pub log: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeInfo {
    pub run: AgentRun,
    pub repository: String,
    pub cleanup_state: String,
    pub message: String,
    pub dirty: Option<bool>,
    pub disk_bytes: Option<u64>,
}
