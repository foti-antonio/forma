use crate::{domain::*, service::Service};
use rusqlite::{params, OptionalExtension, TransactionBehavior};

impl Service {
    pub fn set_theme(&mut self, theme: &str) -> Result<Snapshot> {
        if !["light", "dark", "system"].contains(&theme) {
            return Err(AppError::validation("Choose a supported appearance."));
        }
        self.repo
            .conn
            .execute("UPDATE app_state SET theme=?1 WHERE singleton=1", [theme])?;
        self.snapshot()
    }

    pub fn project_view(&self, project: &str) -> Result<String> {
        id(project)?;
        self.repo.project_workspace(project)?;
        Ok(self
            .repo
            .conn
            .query_row(
                "SELECT view FROM project_views WHERE project_id=?1",
                [project],
                |r| r.get(0),
            )
            .optional()?
            .unwrap_or_else(|| "list".into()))
    }
    pub fn set_project_view(&mut self, project: &str, view: &str) -> Result<()> {
        id(project)?;
        self.repo.project_workspace(project)?;
        if !["list", "board"].contains(&view) {
            return Err(AppError::validation("Choose Board or List."));
        }
        self.repo.conn.execute("INSERT INTO project_views(project_id,view) VALUES(?1,?2) ON CONFLICT(project_id) DO UPDATE SET view=excluded.view", params![project,view])?;
        Ok(())
    }
    /// An anchor denotes insertion before a canonical issue, never a filtered index.
    /// A null anchor appends. IMMEDIATE also serializes separate DB connections.
    pub fn move_issue(
        &mut self,
        project: &str,
        key: &str,
        status: &str,
        before: Option<&str>,
    ) -> Result<()> {
        self.move_issue_relative(project, key, status, before, false)
    }
    pub fn move_issue_relative(
        &mut self,
        project: &str,
        key: &str,
        status: &str,
        before: Option<&str>,
        after: bool,
    ) -> Result<()> {
        id(project)?;
        id(key)?;
        if !["backlog", "todo", "in_progress", "done", "cancelled"].contains(&status) {
            return Err(AppError::validation("Choose a supported status."));
        }
        if let Some(anchor) = before {
            id(anchor)?;
        }
        let tx = self
            .repo
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (old_status, archived, workspace): (String,bool,String) = tx.query_row(
            "SELECT i.status,i.archived,p.workspace_id FROM issues i JOIN projects p ON p.id=i.project_id WHERE i.id=?1 AND i.project_id=?2",
            params![key,project], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional()?.ok_or_else(AppError::missing)?;
        if before == Some(key) {
            return Ok(());
        }
        // Normalize both complete columns (including hidden filtered cards) deterministically.
        for column in [&old_status, &status.to_string()] {
            tx.execute("WITH ranked AS MATERIALIZED (SELECT id,row_number() OVER(ORDER BY position,number,id)-1 AS n FROM issues WHERE project_id=?1 AND archived=?2 AND status=?3) UPDATE issues SET position=ranked.n FROM ranked WHERE issues.id=ranked.id AND issues.position<>ranked.n",params![project,archived,column])?;
        }
        let insertion: i64 = if let Some(anchor) = before {
            tx.query_row("SELECT position FROM issues WHERE id=?1 AND project_id=?2 AND archived=?3 AND status=?4",params![anchor,project,archived,status], |r|r.get::<_,i64>(0)).optional()?.ok_or_else(|| AppError::validation("The target moved. Refresh and try again."))? + i64::from(after)
        } else {
            tx.query_row("SELECT COALESCE(max(position)+1,0) FROM issues WHERE project_id=?1 AND archived=?2 AND status=?3",params![project,archived,status], |r|r.get(0))?
        };
        tx.execute("UPDATE issues SET position=position+1 WHERE project_id=?1 AND archived=?2 AND status=?3 AND position>=?4",params![project,archived,status,insertion])?;
        let now = chrono::Utc::now().to_rfc3339();
        tx.execute(
            "UPDATE issues SET status=?1,position=?2,updated_at=?3 WHERE id=?4",
            params![status, insertion, now, key],
        )?;
        for column in [&old_status, &status.to_string()] {
            tx.execute("WITH ranked AS MATERIALIZED (SELECT id,row_number() OVER(ORDER BY position,number,id)-1 AS n FROM issues WHERE project_id=?1 AND archived=?2 AND status=?3) UPDATE issues SET position=ranked.n FROM ranked WHERE issues.id=ranked.id AND issues.position<>ranked.n",params![project,archived,column])?;
        }
        tx.execute("INSERT INTO activity_events(id,workspace_id,project_id,issue_id,kind,summary,created_at) VALUES(?1,?2,?3,?4,'issue.moved',?5,?6)",params![uuid::Uuid::new_v4().to_string(),workspace,project,key,format!("Moved to {status}; order updated"),now])?;
        tx.commit()?;
        Ok(())
    }
}
