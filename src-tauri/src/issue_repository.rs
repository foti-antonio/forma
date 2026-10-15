use crate::{domain::*, repository::Repository};
use rusqlite::{params, OptionalExtension};
const COLUMNS: &str = "i.id,i.project_id,'F'||k.number||'-'||i.number,i.number,i.title,i.description,i.status,i.priority,i.due_date,i.parent_id,i.position,i.archived,i.created_at,i.updated_at,(SELECT count(*) FROM issues c WHERE c.parent_id=i.id),(SELECT count(*) FROM issues c WHERE c.parent_id=i.id AND c.status='done')";
fn row(r: &rusqlite::Row<'_>) -> rusqlite::Result<Issue> {
    Ok(Issue {
        id: r.get(0)?,
        project_id: r.get(1)?,
        identifier: r.get(2)?,
        number: r.get(3)?,
        title: r.get(4)?,
        description: r.get(5)?,
        status: r.get(6)?,
        priority: r.get(7)?,
        due_date: r.get(8)?,
        parent_id: r.get(9)?,
        position: r.get(10)?,
        archived: r.get(11)?,
        created_at: r.get(12)?,
        updated_at: r.get(13)?,
        label_ids: vec![],
        subtask_total: r.get(14)?,
        subtask_done: r.get(15)?,
    })
}
impl Repository {
    fn issue_labels(&self, mut issue: Issue) -> Result<Issue> {
        issue.label_ids = self
            .conn
            .prepare("SELECT label_id FROM issue_labels WHERE issue_id=?1 ORDER BY label_id")?
            .query_map([&issue.id], |r| r.get(0))?
            .collect::<std::result::Result<_, _>>()?;
        Ok(issue)
    }
    pub fn issue(&self, project: &str, key: &str) -> Result<Issue> {
        let issue = self.conn.query_row(&format!("SELECT {COLUMNS} FROM issues i JOIN project_keys k ON k.project_id=i.project_id WHERE i.id=?1 AND i.project_id=?2"), params![key,project], row).optional()?.ok_or_else(AppError::missing)?;
        self.issue_labels(issue)
    }
    pub fn issues(&self, q: &IssueQuery) -> Result<IssuePage> {
        let order = match q.sort.as_str() {
            "position" => "i.position ASC,i.number ASC,i.id ASC",
            "newest" => "i.created_at DESC,i.id ASC",
            "updated" => "i.updated_at DESC,i.id ASC",
            "due" => "i.due_date IS NULL,i.due_date ASC,i.number ASC,i.id ASC",
            "priority" => "CASE i.priority WHEN 'urgent' THEN 0 WHEN 'high' THEN 1 WHEN 'medium' THEN 2 WHEN 'low' THEN 3 ELSE 4 END,i.position ASC,i.number ASC,i.id ASC",
            _ => return Err(AppError::validation("Choose a supported sort order.")),
        };
        let base_filter = "i.project_id=?1 AND i.archived=?2 AND (?3 IS NULL OR i.status=?3) AND (?4 IS NULL OR i.priority=?4) AND (?5='' OR instr(lower(i.title||' '||i.description||' F'||k.number||'-'||i.number),lower(?5))>0) AND (?8 IS NULL OR EXISTS(SELECT 1 FROM issue_labels il WHERE il.issue_id=i.id AND il.label_id=?8)) AND (?9 IS NULL OR i.due_date>=?9) AND (?10 IS NULL OR i.due_date<=?10) AND (NOT ?11 OR i.due_date IS NULL)";
        let filter = if q.status.is_some() {
            base_filter.replace("(?3 IS NULL OR i.status=?3)", "i.status=?3")
        } else {
            base_filter.to_owned()
        };
        let args = params![
            q.project_id,
            q.archived,
            q.status,
            q.priority,
            q.search,
            q.limit,
            q.offset,
            q.label_id,
            q.due_from,
            q.due_to,
            q.undated
        ];
        let sql = format!("SELECT {COLUMNS} FROM issues i JOIN project_keys k ON k.project_id=i.project_id WHERE {filter} ORDER BY {order} LIMIT ?6 OFFSET ?7");
        let raw = self
            .conn
            .prepare(&sql)?
            .query_map(args, row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let issues = raw
            .into_iter()
            .map(|i| self.issue_labels(i))
            .collect::<Result<Vec<_>>>()?;
        let total = self.conn.query_row(&format!("SELECT count(*) FROM issues i JOIN project_keys k ON k.project_id=i.project_id WHERE {filter}"), params![q.project_id,q.archived,q.status,q.priority,q.search,q.limit,q.offset,q.label_id,q.due_from,q.due_to,q.undated], |r| r.get(0))?;
        let (project_total,done) = self.conn.query_row("SELECT count(*),COALESCE(sum(status='done'),0) FROM issues WHERE project_id=?1 AND archived=?2", params![q.project_id,q.archived], |r| Ok((r.get(0)?,r.get(1)?)))?;
        Ok(IssuePage {
            issues,
            total,
            project_total,
            done,
        })
    }
    pub fn labels(&self, workspace: &str) -> Result<Vec<Label>> {
        Ok(self.conn.prepare("SELECT id,workspace_id,name,color,created_at,updated_at FROM labels WHERE workspace_id=?1 ORDER BY lower(name),id")?.query_map([workspace], |r| Ok(Label {id:r.get(0)?,workspace_id:r.get(1)?,name:r.get(2)?,color:r.get(3)?,created_at:r.get(4)?,updated_at:r.get(5)?}))?.collect::<std::result::Result<_,_>>()?)
    }
    pub fn activity(&self, issue: &str, offset: i64, limit: i64) -> Result<ActivityPage> {
        let events = self.conn.prepare("SELECT id,kind,summary,created_at FROM activity_events WHERE issue_id=?1 ORDER BY created_at DESC,id DESC LIMIT ?2 OFFSET ?3")?.query_map(params![issue,limit,offset], |r| Ok(Activity { id:r.get(0)?,kind:r.get(1)?,summary:r.get(2)?,created_at:r.get(3)? }))?.collect::<std::result::Result<_,_>>()?;
        let total = self.conn.query_row(
            "SELECT count(*) FROM activity_events WHERE issue_id=?1",
            [issue],
            |r| r.get(0),
        )?;
        Ok(ActivityPage { events, total })
    }
}
impl Repository {
    pub fn subtasks(&self, project: &str, key: &str, offset: i64, limit: i64) -> Result<IssuePage> {
        let sql=format!("SELECT {COLUMNS} FROM issues i JOIN project_keys k ON k.project_id=i.project_id WHERE i.project_id=?1 AND i.parent_id=?2 ORDER BY i.position,i.number,i.id LIMIT ?3 OFFSET ?4");
        let raw = self
            .conn
            .prepare(&sql)?
            .query_map(params![project, key, limit, offset], row)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let issues = raw
            .into_iter()
            .map(|i| self.issue_labels(i))
            .collect::<Result<Vec<_>>>()?;
        let total = self.conn.query_row(
            "SELECT count(*) FROM issues WHERE project_id=?1 AND parent_id=?2",
            params![project, key],
            |r| r.get(0),
        )?;
        Ok(IssuePage {
            issues,
            total,
            project_total: total,
            done: 0,
        })
    }
}
