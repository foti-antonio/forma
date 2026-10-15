use crate::{domain::*, git_engine, service::Service};
use std::path::Path;
fn page(offset: usize, limit: usize) -> Result<()> {
    if offset > 10000 || !(1..=100).contains(&limit) {
        return Err(AppError::validation(
            "Git pages require a limit of 1–100 and offset of 0–10,000.",
        ));
    }
    Ok(())
}
impl Service {
    pub fn git_connection(&self, project: &str) -> Result<Option<GitRepository>> {
        id(project)?;
        self.repo.project_workspace(project)?;
        self.repo.git_connection(project)
    }
    /// Only called with a native directory picker result, never an IPC path.
    pub fn git_connect(&mut self, project: &str, path: &Path) -> Result<GitRepository> {
        id(project)?;
        self.repo.project_workspace(project)?;
        let path = path
            .canonicalize()
            .map_err(|_| AppError::validation("Select an accessible repository directory."))?;
        if !path.is_dir() {
            return Err(AppError::validation("Select a repository directory."));
        }
        let repo = git_engine::open(&path)?;
        // Validate objects and index before persisting. Worktrees and bare repositories use the same reader.
        git_engine::overview(&repo, 0, 1)?;
        let root = repo
            .workdir()
            .unwrap_or(repo.path())
            .canonicalize()
            .map_err(|_| AppError::validation("Repository path is unavailable."))?;
        let stored = root
            .to_str()
            .ok_or_else(|| AppError::validation("Repository path must be valid Unicode."))?;
        if stored.chars().any(char::is_control) {
            return Err(AppError::validation(
                "Repository paths cannot contain control characters.",
            ));
        }
        let name = root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Repository");
        self.repo.git_connect(project, stored, name)
    }
    pub fn git_disconnect(&self, project: &str) -> Result<()> {
        self.git_connection(project)?;
        self.repo.git_disconnect(project)
    }
    fn git_open(&self, project: &str, repository: &str) -> Result<git2::Repository> {
        id(project)?;
        id(repository)?;
        let record = self.repo.git_repository(project, repository)?;
        if !record.connected || !record.trusted {
            return Err(AppError::validation("Select the repository folder again to enable local access. Restored paths are never opened automatically."));
        }
        git_engine::open(Path::new(&record.path))
    }
    pub fn git_overview(
        &self,
        project: &str,
        repository: &str,
        offset: usize,
        limit: usize,
    ) -> Result<GitOverview> {
        page(offset, limit)?;
        git_engine::overview(&self.git_open(project, repository)?, offset, limit)
    }
    pub fn git_branches(
        &self,
        project: &str,
        repository: &str,
        search: &str,
        offset: usize,
        limit: usize,
    ) -> Result<GitBranchPage> {
        page(offset, limit)?;
        if search.chars().count() > 200 {
            return Err(AppError::validation(
                "Branch search is limited to 200 characters.",
            ));
        }
        git_engine::branches(&self.git_open(project, repository)?, search, offset, limit)
    }
    pub fn git_links(&self, project: &str, issue: &str) -> Result<Vec<GitLink>> {
        self.issue(project, issue)?;
        self.repo.git_links(issue)
    }
    pub fn git_link(&self, mut input: GitLinkInput) -> Result<Vec<GitLink>> {
        self.issue(&input.project_id, &input.issue_id)?;
        if !input.confirmed {
            return Err(AppError::validation(
                "Confirm this Git association before saving.",
            ));
        }
        if !["branch", "commit"].contains(&input.kind.as_str()) || input.target.len() > 1024 {
            return Err(AppError::validation(
                "Choose a local branch or full commit hash.",
            ));
        }
        if input.kind == "branch" {
            if !git2::Reference::is_valid_name(&format!("refs/heads/{}", input.target))
                || input.target.is_empty()
            {
                return Err(AppError::validation("Invalid local branch name."));
            }
        } else if input.target.len() != 40 || !input.target.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(AppError::validation(
                "Enter a full 40-character commit hash.",
            ));
        }
        if self.repo.git_links(&input.issue_id)?.len() >= 200 {
            return Err(AppError::validation(
                "An issue supports at most 200 Git links. Remove a link first.",
            ));
        }
        let repo = self.git_open(&input.project_id, &input.repository_id)?;
        let commit = git_engine::target(&repo, &input.kind, &input.target)?;
        if input.kind == "commit" {
            input.target = commit.hash.clone();
        }
        self.repo.git_save_link(&input, &commit)?;
        self.repo.git_links(&input.issue_id)
    }
    pub fn git_unlink(&self, project: &str, issue: &str, link: &str) -> Result<Vec<GitLink>> {
        self.issue(project, issue)?;
        id(link)?;
        self.repo.git_unlink(issue, link)?;
        self.repo.git_links(issue)
    }
}
