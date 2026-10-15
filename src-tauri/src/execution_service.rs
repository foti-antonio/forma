//! Explicit approval queue. Durable snapshots are data, never authorization after restart/restore.
use crate::{
    agent_provider as engine, agent_service::Prepared, agent_types::*, domain::*, service::Service,
};
use rusqlite::params;

impl Service {
    pub fn agent_concurrency(&self) -> Result<usize> {
        Ok(self.repo.conn.query_row(
            "SELECT max_concurrent FROM agent_preferences WHERE singleton=1",
            [],
            |r| r.get(0),
        )?)
    }
    pub fn agent_set_concurrency(&self, maximum: usize) -> Result<()> {
        if ![1, 2].contains(&maximum) {
            return Err(AppError::validation(
                "Maximum concurrent runs must be 1 or 2.",
            ));
        }
        self.repo.conn.execute(
            "UPDATE agent_preferences SET max_concurrent=?1 WHERE singleton=1",
            [maximum],
        )?;
        Ok(())
    }
    pub fn agent_enqueue(&mut self, token: &str, approved: bool, elevated: bool) -> Result<String> {
        if !approved {
            return Err(engine::error("Explicit task approval is required."));
        }
        let prepared = self
            .agents
            .pending
            .remove(token)
            .ok_or_else(|| engine::error("Preview expired. Preview and approve again."))?;
        if chrono::Utc::now().timestamp() - prepared.created > 300 {
            return Err(engine::error("Approval expired."));
        }
        if prepared.preview.provider == "claude" && !elevated {
            return Err(engine::error(
                "Claude requires separate consent to host filesystem risk.",
            ));
        }
        let count: usize = self.repo.conn.query_row(
            "SELECT count(*) FROM agent_queue WHERE status='Approved'",
            [],
            |r| r.get(0),
        )?;
        if count >= 100 {
            return Err(engine::error("Approved queue limit is 100 tasks."));
        }
        if self.repo.conn.prepare("SELECT 1 FROM agent_queue WHERE issue_id=?1 AND status IN ('Approved','Launching')")?.exists([&prepared.issue.id])? {
            return Err(engine::error("This issue already has an approved queued task."));
        }
        let payload = serde_json::to_string(&prepared)
            .map_err(|_| engine::error("Cannot encode task snapshot."))?;
        let id = uuid::Uuid::new_v4().to_string();
        let expires = chrono::DateTime::from_timestamp(prepared.created + 300, 0)
            .ok_or_else(|| engine::error("Invalid approval timestamp."))?
            .to_rfc3339();
        self.repo.conn.execute("INSERT INTO agent_queue(id,issue_id,provider,payload,approved_at,expires_at,position,status) VALUES(?1,?2,?3,?4,?5,?6,(SELECT coalesce(max(position),0)+1 FROM agent_queue),'Approved')", params![id,prepared.issue.id,prepared.preview.provider,payload,chrono::Utc::now().to_rfc3339(),expires])?;
        Ok(id)
    }
    pub fn agent_queue(&self) -> Result<Vec<QueuedTask>> {
        self.repo.conn.execute("UPDATE agent_queue SET status='Invalidated',payload='',message='Approval expired. Preview and approve again.' WHERE status='Approved' AND julianday(expires_at)<=julianday('now')", [])?;
        Ok(self.repo.conn.prepare("SELECT q.id,q.issue_id,q.provider,q.approved_at,q.expires_at,q.position,q.status,q.message,q.run_id,i.title FROM agent_queue q JOIN issues i ON i.id=q.issue_id ORDER BY CASE WHEN q.status='Approved' THEN 0 ELSE 1 END,q.position,q.id LIMIT 100")?.query_map([], |r| Ok(QueuedTask {id:r.get(0)?,issue_id:r.get(1)?,provider:r.get(2)?,approved_at:r.get(3)?,expires_at:r.get(4)?,position:r.get(5)?,status:r.get(6)?,message:r.get(7)?,run_id:r.get(8)?,title:r.get(9)?}))?.collect::<std::result::Result<_,_>>()?)
    }
    pub fn agent_queue_cancel(&self, task: &str) -> Result<()> {
        id(task)?;
        self.repo.conn.execute("UPDATE agent_queue SET status='Cancelled',payload='',message='Cancelled by user.' WHERE id=?1 AND status='Approved'", [task])?;
        Ok(())
    }
    pub fn agent_queue_move(&mut self, task: &str, before: Option<&str>) -> Result<()> {
        id(task)?;
        if let Some(before) = before {
            id(before)?;
        }
        let mut ids = self
            .repo
            .conn
            .prepare("SELECT id FROM agent_queue WHERE status='Approved' ORDER BY position,id")?
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if !ids.iter().any(|x| x == task) {
            return Err(AppError::missing());
        }
        if before == Some(task) {
            return Ok(());
        }
        ids.retain(|x| x != task);
        let at = match before {
            Some(b) => ids
                .iter()
                .position(|x| x == b)
                .ok_or_else(AppError::missing)?,
            None => ids.len(),
        };
        ids.insert(at, task.into());
        let tx = self.repo.conn.transaction()?;
        for (position, id) in ids.iter().enumerate() {
            tx.execute(
                "UPDATE agent_queue SET position=?2 WHERE id=?1",
                params![id, position],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
    pub fn agent_queue_begin(&mut self) -> Result<Vec<AgentRun>> {
        if self.agents.queue_session.is_some() {
            return Err(engine::error(
                "Queue is already dispatching. Stop it before starting another session.",
            ));
        }
        self.agents.queue_generation += 1;
        self.agents.queue_session = Some(
            self.agent_queue()?
                .into_iter()
                .filter(|t| t.status == "Approved")
                .map(|t| t.id)
                .collect(),
        );
        let result = self.agent_start_queue(true);
        if result.is_err() {
            self.agent_queue_stop();
        }
        result
    }
    pub fn agent_queue_stop(&mut self) {
        self.agents.queue_session = None;
    }
    pub fn agent_queue_tick(&mut self) -> Result<bool> {
        if self.agents.queue_session.is_none() {
            return Ok(false);
        }
        self.agent_start_queue(true)?;
        let eligible = self.agent_queue()?.iter().any(|t| {
            t.status == "Approved"
                && self
                    .agents
                    .queue_session
                    .as_ref()
                    .is_some_and(|s| s.contains(&t.id))
        });
        if !eligible {
            self.agents.queue_session = None;
        }
        Ok(eligible)
    }
    /// Dispatch is enabled only by an explicit Start action; session membership is frozen.
    pub fn agent_start_queue(&mut self, all: bool) -> Result<Vec<AgentRun>> {
        let mut runs = Vec::new();
        let tasks = self.agent_queue()?;
        for task in tasks.into_iter().filter(|t| t.status == "Approved") {
            if self
                .agents
                .queue_session
                .as_ref()
                .is_some_and(|s| !s.contains(&task.id))
            {
                continue;
            }
            let running: usize = self.repo.conn.query_row(
                "SELECT (SELECT count(*) FROM agent_runs WHERE status IN ('Queued','Running'))+(SELECT count(*) FROM independent_tests WHERE status='Running')",
                [],
                |r| r.get(0),
            )?;
            if running >= self.agent_concurrency()? {
                break;
            }
            // Keep tasks for active issues queued instead of invalidating their approval.
            if self
                .repo
                .conn
                .prepare(
                    "SELECT 1 FROM agent_runs WHERE issue_id=?1 AND status IN ('Queued','Running')",
                )?
                .exists([&task.issue_id])?
            {
                continue;
            }
            let payload: String = self.repo.conn.query_row(
                "SELECT payload FROM agent_queue WHERE id=?1 AND status='Approved'",
                [&task.id],
                |r| r.get(0),
            )?;
            self.repo.conn.execute(
                "UPDATE agent_queue SET status='Launching' WHERE id=?1",
                [&task.id],
            )?;
            let launched = serde_json::from_str::<Prepared>(&payload)
                .map_err(|_| engine::error("Invalid queued snapshot; approve again."))
                .and_then(|p| self.agent_launch(p));
            match launched {
                Ok(run) => {
                    self.repo.conn.execute(
                        "UPDATE agent_queue SET status='Started',payload='',run_id=?2 WHERE id=?1",
                        params![task.id, run.id],
                    )?;
                    runs.push(run);
                }
                Err(e) => {
                    self.repo.conn.execute("UPDATE agent_queue SET status='Invalidated',payload='',message=?2 WHERE id=?1", params![task.id,e.message])?;
                }
            }
            if !all {
                break;
            }
        }
        Ok(runs)
    }
}
