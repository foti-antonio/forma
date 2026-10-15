use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AppError {
    pub code: &'static str,
    pub message: String,
}
pub type Result<T> = std::result::Result<T, AppError>;
impl AppError {
    pub fn validation(message: &str) -> Self {
        Self {
            code: "validation",
            message: message.into(),
        }
    }
    pub fn missing() -> Self {
        Self {
            code: "not_found",
            message: "This item no longer exists. Refresh and try again.".into(),
        }
    }
}
impl From<rusqlite::Error> for AppError {
    fn from(error: rusqlite::Error) -> Self {
        eprintln!("Database error: {error}");
        Self {
            code: "database",
            message: "Forma could not save or read local data. Please try again.".into(),
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub status: String,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectInput {
    pub workspace_id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub status: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub theme: Option<String>,
    pub workspaces: Vec<Workspace>,
    pub active_workspace_id: Option<String>,
    pub projects: Vec<Project>,
}
pub fn name(value: &str, max: usize) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max || value.chars().any(char::is_control) {
        return Err(AppError::validation(&format!(
            "Name must contain 1–{max} characters and no control characters."
        )));
    }
    Ok(value.into())
}
pub fn id(value: &str) -> Result<()> {
    uuid::Uuid::parse_str(value).map_err(|_| AppError::validation("Invalid item identifier."))?;
    Ok(())
}
impl ProjectInput {
    pub fn validate(&mut self) -> Result<()> {
        id(&self.workspace_id)?;
        self.name = name(&self.name, 100)?;
        if self.description.chars().count() > 4000 {
            return Err(AppError::validation(
                "Description must be 4,000 characters or fewer.",
            ));
        }
        if !["layers", "code", "rocket", "box", "globe", "sparkles"].contains(&self.icon.as_str()) {
            return Err(AppError::validation("Choose a supported project icon."));
        }
        if self.color.len() != 7
            || !self.color.starts_with('#')
            || !self.color[1..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(AppError::validation("Choose a valid hex color."));
        }
        if !["planned", "active", "paused", "completed"].contains(&self.status.as_str()) {
            return Err(AppError::validation("Choose a supported project status."));
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IssueInput {
    pub project_id: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub parent_id: Option<String>,
    pub label_ids: Vec<String>,
    pub position: i64,
}
impl IssueInput {
    pub fn validate(&mut self) -> Result<()> {
        id(&self.project_id)?;
        self.title = name(&self.title, 200)?;
        if self.description.chars().count() > 50000 {
            return Err(AppError::validation(
                "Description must be 50,000 characters or fewer.",
            ));
        }
        if !["backlog", "todo", "in_progress", "done", "cancelled"].contains(&self.status.as_str())
        {
            return Err(AppError::validation("Choose a supported issue status."));
        }
        if !["urgent", "high", "medium", "low", "none"].contains(&self.priority.as_str()) {
            return Err(AppError::validation("Choose a supported priority."));
        }
        if let Some(date) = &self.due_date {
            if date.len() != 10 || chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
                return Err(AppError::validation(
                    "Due date must be a valid YYYY-MM-DD date.",
                ));
            }
        }
        if self.position < 0 || self.position > 1_000_000_000 {
            return Err(AppError::validation(
                "Order must be between 0 and 1,000,000,000.",
            ));
        }
        if let Some(parent) = &self.parent_id {
            id(parent)?;
        }
        if self.label_ids.len() > 100 {
            return Err(AppError::validation("Choose at most 100 labels."));
        }
        let mut unique = std::collections::HashSet::new();
        for label in &self.label_ids {
            id(label)?;
            if !unique.insert(label) {
                return Err(AppError::validation("A label can only be assigned once."));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
    pub id: String,
    pub project_id: String,
    pub identifier: String,
    pub number: i64,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<String>,
    pub parent_id: Option<String>,
    pub label_ids: Vec<String>,
    pub subtask_total: i64,
    pub subtask_done: i64,
    pub position: i64,
    pub archived: bool,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Label {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub color: String,
    pub created_at: String,
    pub updated_at: String,
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LabelInput {
    pub workspace_id: String,
    pub name: String,
    pub color: String,
}
impl LabelInput {
    pub fn validate(&mut self) -> Result<()> {
        id(&self.workspace_id)?;
        self.name = name(&self.name, 40)?;
        if self.color.len() != 7
            || !self.color.starts_with('#')
            || !self.color[1..].bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(AppError::validation("Choose a valid hex color."));
        }
        Ok(())
    }
}
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IssueQuery {
    #[serde(default)]
    pub label_id: Option<String>,
    #[serde(default)]
    pub due_from: Option<String>,
    #[serde(default)]
    pub due_to: Option<String>,
    #[serde(default)]
    pub undated: bool,
    pub project_id: String,
    pub search: String,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub archived: bool,
    pub sort: String,
    pub offset: i64,
    pub limit: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuePage {
    pub issues: Vec<Issue>,
    pub total: i64,
    pub project_total: i64,
    pub done: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub kind: String,
    pub summary: String,
    pub created_at: String,
}
#[derive(Debug, Serialize)]
pub struct ActivityPage {
    pub events: Vec<Activity>,
    pub total: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitRepository {
    pub id: String,
    pub project_id: String,
    pub path: String,
    pub name: String,
    pub connected: bool,
    pub trusted: bool,
}
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitCommit {
    pub hash: String,
    pub message: String,
    pub author: String,
    pub committed_at: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranch {
    pub name: String,
    pub active: bool,
    pub commit: GitCommit,
}
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    pub modified: usize,
    pub added: usize,
    pub deleted: usize,
    pub untracked: usize,
    pub conflicted: usize,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitOverview {
    pub branch: String,
    pub bare: bool,
    pub status: Option<GitStatus>,
    pub commits: Vec<GitCommit>,
    pub has_more: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitBranchPage {
    pub branches: Vec<GitBranch>,
    pub has_more: bool,
    pub truncated: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitLink {
    pub id: String,
    pub repository_id: String,
    pub repository_name: String,
    pub repository_path: String,
    pub connected: bool,
    pub kind: String,
    pub target: String,
    pub commit: GitCommit,
    pub created_at: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GitLinkInput {
    pub project_id: String,
    pub issue_id: String,
    pub repository_id: String,
    pub kind: String,
    pub target: String,
    pub confirmed: bool,
}
