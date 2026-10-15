use crate::{agent_types::*, domain::*, repository::Repository};
use rusqlite::params;
impl Repository {
    pub(crate) fn agent_settings(&self) -> Result<Vec<AgentSettings>> {
        Ok(self.conn.prepare("SELECT provider,executable,trusted,max_runtime,log_limit,policy,require_os_isolation FROM agent_settings ORDER BY provider")?.query_map([], |r| Ok(AgentSettings {provider:r.get(0)?, executable:r.get(1)?, trusted:r.get(2)?, max_runtime:r.get(3)?, log_limit:r.get(4)?,policy:r.get(5)?,require_os_isolation:r.get(6)?}))?.collect::<std::result::Result<_,_>>()?)
    }
    pub(crate) fn agent_save_settings(&self, s: &AgentSettings) -> Result<()> {
        self.conn.execute("UPDATE agent_settings SET executable=?2,trusted=?3,max_runtime=?4,log_limit=?5,policy=?6,require_os_isolation=?7 WHERE provider=?1", params![s.provider,s.executable,s.trusted,s.max_runtime,s.log_limit,s.policy,s.require_os_isolation])?;
        Ok(())
    }
    pub(crate) fn agent_runs(&self, issue: &str, offset: usize) -> Result<Vec<AgentRun>> {
        Ok(self.conn.prepare("SELECT id,issue_id,repository_id,provider,worktree,branch,base_commit,status,started_at,ended_at,exit_code,result,log,review,trusted,(SELECT 'F'||k.number||'-'||i.number FROM issues i JOIN project_keys k ON k.project_id=i.project_id WHERE i.id=agent_runs.issue_id),(SELECT title FROM issues WHERE id=agent_runs.issue_id) FROM agent_runs WHERE issue_id=?1 ORDER BY started_at DESC,id LIMIT 20 OFFSET ?2")?.query_map(params![issue,offset], row)?.collect::<std::result::Result<_,_>>()?)
    }
    pub(crate) fn agent_recent(&self, offset: usize) -> Result<Vec<AgentRun>> {
        Ok(self.conn.prepare("SELECT id,issue_id,repository_id,provider,worktree,branch,base_commit,status,started_at,ended_at,exit_code,result,log,review,trusted,(SELECT 'F'||k.number||'-'||i.number FROM issues i JOIN project_keys k ON k.project_id=i.project_id WHERE i.id=agent_runs.issue_id),(SELECT title FROM issues WHERE id=agent_runs.issue_id) FROM agent_runs ORDER BY started_at DESC,id LIMIT 20 OFFSET ?1")?.query_map([offset], row)?.collect::<std::result::Result<_,_>>()?)
    }
    pub(crate) fn agent_run(&self, id: &str) -> Result<AgentRun> {
        use rusqlite::OptionalExtension;
        self.conn.query_row("SELECT id,issue_id,repository_id,provider,worktree,branch,base_commit,status,started_at,ended_at,exit_code,result,log,review,trusted,(SELECT 'F'||k.number||'-'||i.number FROM issues i JOIN project_keys k ON k.project_id=i.project_id WHERE i.id=agent_runs.issue_id),(SELECT title FROM issues WHERE id=agent_runs.issue_id) FROM agent_runs WHERE id=?1", [id],row).optional()?.ok_or_else(AppError::missing)
    }
    pub(crate) fn agent_insert(&self, r: &AgentRun) -> Result<()> {
        self.conn.execute("INSERT INTO agent_runs(id,issue_id,repository_id,provider,worktree,branch,base_commit,status,started_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",params![r.id,r.issue_id,r.repository_id,r.provider,r.worktree,r.branch,r.base_commit,r.status,r.started_at])?;
        Ok(())
    }
    pub(crate) fn agent_update(
        &self,
        run: &str,
        status: &str,
        exit: Option<i32>,
        result: &str,
        log: &str,
    ) -> Result<()> {
        self.conn.execute("UPDATE agent_runs SET status=?2,exit_code=?3,result=?4,log=?5,ended_at=CASE WHEN ?2 IN ('Queued','Running') THEN NULL ELSE ?6 END WHERE id=?1", params![run,status,exit,result,log,chrono::Utc::now().to_rfc3339()])?;
        Ok(())
    }
    pub(crate) fn agent_recover(&self) -> Result<()> {
        self.conn.execute("UPDATE agent_runs SET cleanup_state='Failed',cleanup_message='Cleanup interrupted. Inspect the retained path and retry explicitly.' WHERE cleanup_state='Removing'", [])?;
        self.conn.execute("UPDATE independent_tests SET status='Interrupted',ended_at=?1,result='Forma stopped; tests were not resumed.' WHERE status='Running'", [chrono::Utc::now().to_rfc3339()])?;
        self.conn.execute("UPDATE agent_queue SET status='Invalidated',payload='',message='Forma restarted; preview and approve again.' WHERE status IN ('Approved','Launching')", [])?;
        self.conn.execute("UPDATE agent_runs SET status='Interrupted',ended_at=?1,result='Forma stopped before execution completed. Inspect the retained worktree; execution was not resumed.' WHERE status IN ('Queued','Running')",[chrono::Utc::now().to_rfc3339()])?;
        Ok(())
    }
    pub(crate) fn agent_active(&self) -> Result<bool> {
        Ok(self
            .conn
            .prepare("SELECT 1 FROM agent_runs WHERE status IN ('Queued','Running') LIMIT 1")?
            .exists([])?)
    }
}
fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<AgentRun> {
    Ok(AgentRun {
        issue_identifier: r.get(15)?,
        issue_title: r.get(16)?,
        id: r.get(0)?,
        issue_id: r.get(1)?,
        repository_id: r.get(2)?,
        provider: r.get(3)?,
        worktree: r.get(4)?,
        branch: r.get(5)?,
        base_commit: r.get(6)?,
        status: r.get(7)?,
        started_at: r.get(8)?,
        ended_at: r.get(9)?,
        exit_code: r.get(10)?,
        result: r.get(11)?,
        log: r.get(12)?,
        review: r.get(13)?,
        trusted: r.get(14)?,
    })
}
