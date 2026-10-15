use crate::{domain::*, repository::Repository};
use rusqlite::{params, OptionalExtension};
impl Repository {
    pub(crate) fn git_connection(&self, project: &str) -> Result<Option<GitRepository>> {
        Ok(self.conn.query_row("SELECT id,project_id,path,name,connected,trusted FROM git_repositories WHERE project_id=?1 AND connected=1", [project], git_row).optional()?)
    }
    pub(crate) fn git_repository(&self, project: &str, repository: &str) -> Result<GitRepository> {
        self.conn.query_row("SELECT id,project_id,path,name,connected,trusted FROM git_repositories WHERE project_id=?1 AND id=?2", params![project,repository], git_row).optional()?.ok_or_else(AppError::missing)
    }
    pub(crate) fn git_connect(
        &mut self,
        project: &str,
        path: &str,
        name: &str,
    ) -> Result<GitRepository> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "UPDATE git_repositories SET connected=0 WHERE project_id=?1",
            [project],
        )?;
        tx.execute("INSERT INTO git_repositories(id,project_id,path,name,created_at) VALUES(?1,?2,?3,?4,?5) ON CONFLICT(project_id,path) DO UPDATE SET connected=1,trusted=1,name=excluded.name", params![uuid::Uuid::new_v4().to_string(),project,path,name,chrono::Utc::now().to_rfc3339()])?;
        tx.commit()?;
        self.git_connection(project)?.ok_or_else(AppError::missing)
    }
    pub(crate) fn git_disconnect(&self, project: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE git_repositories SET connected=0 WHERE project_id=?1",
            [project],
        )?;
        Ok(())
    }
    pub(crate) fn git_links(&self, issue: &str) -> Result<Vec<GitLink>> {
        Ok(self.conn.prepare("SELECT l.id,l.repository_id,r.name,r.path,r.connected,l.kind,l.target,l.commit_hash,l.message,l.author,l.committed_at,l.created_at FROM git_links l JOIN git_repositories r ON r.id=l.repository_id WHERE l.issue_id=?1 ORDER BY l.created_at DESC,l.id LIMIT 200")?
        .query_map([issue], |r| Ok(GitLink {
            id:r.get(0)?,repository_id:r.get(1)?,repository_name:r.get(2)?,repository_path:r.get(3)?,connected:r.get(4)?,kind:r.get(5)?,target:r.get(6)?,
            commit:GitCommit { hash:r.get(7)?,message:r.get(8)?,author:r.get(9)?,committed_at:r.get(10)? },created_at:r.get(11)?
        }))?.collect::<std::result::Result<Vec<_>,_>>()?)
    }
    pub(crate) fn git_save_link(&self, input: &GitLinkInput, commit: &GitCommit) -> Result<()> {
        self.conn.execute("INSERT INTO git_links(id,project_id,issue_id,repository_id,kind,target,commit_hash,message,author,committed_at,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11) ON CONFLICT(issue_id,repository_id,kind,target) DO NOTHING", params![uuid::Uuid::new_v4().to_string(),input.project_id,input.issue_id,input.repository_id,input.kind,input.target,commit.hash,commit.message,commit.author,commit.committed_at,chrono::Utc::now().to_rfc3339()])?;
        Ok(())
    }
    pub(crate) fn git_unlink(&self, issue: &str, link: &str) -> Result<()> {
        crate::repository::changed(self.conn.execute(
            "DELETE FROM git_links WHERE id=?1 AND issue_id=?2",
            params![link, issue],
        )?)
    }
}
fn git_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<GitRepository> {
    Ok(GitRepository {
        id: r.get(0)?,
        project_id: r.get(1)?,
        path: r.get(2)?,
        name: r.get(3)?,
        connected: r.get(4)?,
        trusted: r.get(5)?,
    })
}
