use crate::domain::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub struct Repository {
    pub(crate) conn: Connection,
}
impl Repository {
    pub fn open(path: &Path) -> Result<Self> {
        let mut conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        let version: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version > 3 {
            return Err(AppError {
                code: "migration",
                message: "This database was created by a newer Forma version. Please update Forma."
                    .into(),
            });
        }
        if version == 0 {
            let tx = conn.transaction()?;
            tx.execute_batch(include_str!("../migrations/001_initial.sql"))?;
            tx.pragma_update(None, "user_version", 1)?;
            tx.commit()?;
        }
        if version < 2 {
            let tx = conn.transaction()?;
            tx.execute_batch(include_str!("../migrations/002_issues.sql"))?;
            tx.pragma_update(None, "user_version", 2)?;
            tx.commit()?;
        }
        if version < 3 {
            let tx = conn.transaction()?;
            tx.execute_batch(include_str!("../migrations/003_productivity.sql"))?;
            tx.pragma_update(None, "user_version", 3)?;
            tx.commit()?;
        }
        Ok(Self { conn })
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        let workspaces = self
            .conn
            .prepare("SELECT id,name,created_at,updated_at FROM workspaces ORDER BY created_at,id")?
            .query_map([], |r| {
                Ok(Workspace {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    created_at: r.get(2)?,
                    updated_at: r.get(3)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let active_workspace_id: Option<String> = self.conn.query_row(
            "SELECT active_workspace_id FROM app_state WHERE singleton=1",
            [],
            |r| r.get(0),
        )?;
        let projects = self.conn.prepare("SELECT id,workspace_id,name,description,icon,color,status,archived,created_at,updated_at FROM projects WHERE workspace_id=?1 ORDER BY updated_at DESC,id")?
            .query_map([&active_workspace_id], |r| Ok(Project { id:r.get(0)?, workspace_id:r.get(1)?, name:r.get(2)?, description:r.get(3)?, icon:r.get(4)?, color:r.get(5)?, status:r.get(6)?, archived:r.get(7)?, created_at:r.get(8)?, updated_at:r.get(9)? }))?
            .collect::<std::result::Result<Vec<_>,_>>()?;
        let theme =
            self.conn
                .query_row("SELECT theme FROM app_state WHERE singleton=1", [], |r| {
                    r.get(0)
                })?;
        Ok(Snapshot {
            theme,
            workspaces,
            active_workspace_id,
            projects,
        })
    }
    pub fn workspace_exists(&self, id: &str) -> Result<()> {
        if self
            .conn
            .query_row("SELECT 1 FROM workspaces WHERE id=?1", [id], |_| Ok(()))
            .optional()?
            .is_none()
        {
            return Err(AppError::missing());
        }
        Ok(())
    }
    pub fn project_workspace(&self, id: &str) -> Result<String> {
        self.conn
            .query_row("SELECT workspace_id FROM projects WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .optional()?
            .ok_or_else(AppError::missing)
    }
}
pub(crate) fn changed(count: usize) -> Result<()> {
    if count == 0 {
        Err(AppError::missing())
    } else {
        Ok(())
    }
}
pub(crate) fn event(
    tx: &rusqlite::Transaction<'_>,
    workspace: &str,
    project: Option<&str>,
    kind: &str,
    summary: &str,
    now: &str,
) -> Result<()> {
    tx.execute("INSERT INTO activity_events(id,workspace_id,project_id,kind,summary,created_at) VALUES(?1,?2,?3,?4,?5,?6)",
        params![uuid::Uuid::new_v4().to_string(), workspace, project, kind, summary, now])?;
    Ok(())
}
