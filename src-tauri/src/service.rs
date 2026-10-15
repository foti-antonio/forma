use crate::{
    domain::*,
    repository::{changed, event, Repository},
};
use rusqlite::params;
pub struct Service {
    pub(crate) repo: Repository,
    pub(crate) pending_restore: Option<tempfile::NamedTempFile>,
}
impl Service {
    pub fn open(path: &std::path::Path) -> Result<Self> {
        Ok(Self {
            repo: Repository::open(path)?,
            pending_restore: None,
        })
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        self.repo.snapshot()
    }
    pub fn create_workspace(&mut self, value: &str) -> Result<Snapshot> {
        let name = name(value, 80)?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let tx = self.repo.conn.transaction()?;
        tx.execute(
            "INSERT INTO workspaces(id,name,created_at,updated_at) VALUES(?1,?2,?3,?3)",
            params![id, name, now],
        )?;
        tx.execute(
            "UPDATE app_state SET active_workspace_id=?1 WHERE singleton=1",
            [&id],
        )?;
        event(&tx, &id, None, "workspace.created", &name, &now)?;
        tx.commit()?;
        self.snapshot()
    }
    pub fn rename_workspace(&mut self, key: &str, value: &str) -> Result<Snapshot> {
        id(key)?;
        let name = name(value, 80)?;
        let now = chrono::Utc::now().to_rfc3339();
        let tx = self.repo.conn.transaction()?;
        changed(tx.execute(
            "UPDATE workspaces SET name=?1,updated_at=?2 WHERE id=?3",
            params![name, now, key],
        )?)?;
        event(&tx, key, None, "workspace.renamed", &name, &now)?;
        tx.commit()?;
        self.snapshot()
    }
    pub fn select_workspace(&mut self, key: &str) -> Result<Snapshot> {
        id(key)?;
        self.repo.workspace_exists(key)?;
        self.repo.conn.execute(
            "UPDATE app_state SET active_workspace_id=?1 WHERE singleton=1",
            [key],
        )?;
        self.snapshot()
    }
    pub fn delete_workspace(&mut self, key: &str) -> Result<Snapshot> {
        id(key)?;
        let tx = self.repo.conn.transaction()?;
        changed(tx.execute("DELETE FROM workspaces WHERE id=?1", [key])?)?;
        tx.execute("UPDATE app_state SET active_workspace_id=(SELECT id FROM workspaces ORDER BY created_at,id LIMIT 1) WHERE singleton=1 AND active_workspace_id IS NULL",[])?;
        tx.commit()?;
        self.snapshot()
    }
    pub fn save_project(&mut self, key: Option<&str>, mut input: ProjectInput) -> Result<Snapshot> {
        input.validate()?;
        self.repo.workspace_exists(&input.workspace_id)?;
        if let Some(key) = key {
            id(key)?;
            if self.repo.project_workspace(key)? != input.workspace_id {
                return Err(AppError::validation(
                    "A project cannot move between workspaces.",
                ));
            }
        }
        let project_id = key
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let now = chrono::Utc::now().to_rfc3339();
        let tx = self.repo.conn.transaction()?;
        if key.is_some() {
            changed(tx.execute("UPDATE projects SET name=?1,description=?2,icon=?3,color=?4,status=?5,updated_at=?6 WHERE id=?7",
                params![input.name,input.description,input.icon,input.color,input.status,now,project_id])?)?;
        } else {
            tx.execute("INSERT INTO projects(id,workspace_id,name,description,icon,color,status,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8)",
                params![project_id,input.workspace_id,input.name,input.description,input.icon,input.color,input.status,now])?;
        }
        event(
            &tx,
            &input.workspace_id,
            Some(&project_id),
            if key.is_some() {
                "project.updated"
            } else {
                "project.created"
            },
            &input.name,
            &now,
        )?;
        tx.commit()?;
        self.snapshot()
    }
    pub fn archive_project(&mut self, key: &str, archived: bool) -> Result<Snapshot> {
        id(key)?;
        let workspace = self.repo.project_workspace(key)?;
        let now = chrono::Utc::now().to_rfc3339();
        let tx = self.repo.conn.transaction()?;
        changed(tx.execute(
            "UPDATE projects SET archived=?1,updated_at=?2 WHERE id=?3",
            params![archived, now, key],
        )?)?;
        event(
            &tx,
            &workspace,
            Some(key),
            if archived {
                "project.archived"
            } else {
                "project.restored"
            },
            "",
            &now,
        )?;
        tx.commit()?;
        self.snapshot()
    }
    pub fn delete_project(&mut self, key: &str) -> Result<Snapshot> {
        id(key)?;
        let workspace = self.repo.project_workspace(key)?;
        let now = chrono::Utc::now().to_rfc3339();
        let tx = self.repo.conn.transaction()?;
        changed(tx.execute("DELETE FROM projects WHERE id=?1", [key])?)?;
        event(
            &tx,
            &workspace,
            None,
            "project.deleted",
            "Project deleted",
            &now,
        )?;
        tx.commit()?;
        self.snapshot()
    }
}
