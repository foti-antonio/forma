use crate::{domain::*, repository::changed, service::Service};
use rusqlite::{params, OptionalExtension, Transaction};
fn issue_event(
    tx: &Transaction<'_>,
    workspace: &str,
    project: &str,
    issue: &str,
    kind: &str,
    summary: &str,
    now: &str,
) -> Result<()> {
    tx.execute("INSERT INTO activity_events(id,workspace_id,project_id,issue_id,kind,summary,created_at) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![uuid::Uuid::new_v4().to_string(),workspace,project,issue,kind,summary,now])?;
    Ok(())
}
impl Service {
    pub fn issues(&self, q: IssueQuery) -> Result<IssuePage> {
        id(&q.project_id)?;
        self.repo.project_workspace(&q.project_id)?;
        pagination(q.offset, q.limit)?;
        if q.search.chars().count() > 200
            || q.status.as_ref().is_some_and(|s| {
                !["backlog", "todo", "in_progress", "done", "cancelled"].contains(&s.as_str())
            })
            || q.priority
                .as_ref()
                .is_some_and(|s| !["urgent", "high", "medium", "low", "none"].contains(&s.as_str()))
        {
            return Err(AppError::validation("Invalid issue filters."));
        }
        if let Some(label) = &q.label_id {
            id(label)?;
            let workspace = self.repo.project_workspace(&q.project_id)?;
            if !self.labels(&workspace)?.iter().any(|l| &l.id == label) {
                return Err(AppError::validation("Choose a label from this workspace."));
            }
        }
        for date in [&q.due_from, &q.due_to].into_iter().flatten() {
            if date.len() != 10 || chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").is_err() {
                return Err(AppError::validation("Invalid due-date filter."));
            }
        }
        if q.due_from
            .as_ref()
            .zip(q.due_to.as_ref())
            .is_some_and(|(a, b)| a > b)
            || (q.undated && (q.due_from.is_some() || q.due_to.is_some()))
        {
            return Err(AppError::validation(
                "Choose a valid date range or undated issues.",
            ));
        }
        self.repo.issues(&q)
    }
    pub fn issue(&self, project: &str, key: &str) -> Result<Issue> {
        id(project)?;
        id(key)?;
        self.repo.issue(project, key)
    }
    pub fn save_issue(&mut self, key: Option<&str>, mut input: IssueInput) -> Result<Issue> {
        input.validate()?;
        let workspace = self.repo.project_workspace(&input.project_id)?;
        let old = key.map(|k| self.issue(&input.project_id, k)).transpose()?;
        if let Some(parent) = &input.parent_id {
            let p = self.issue(&input.project_id, parent)?;
            if Some(parent.as_str()) == key
                || p.parent_id.is_some()
                || (p.archived && old.as_ref().and_then(|i| i.parent_id.as_ref()) != Some(parent))
            {
                return Err(AppError::validation(
                    "Choose an active top-level issue in this project as parent.",
                ));
            }
            if let Some(k) = key {
                let children: i64 = self.repo.conn.query_row(
                    "SELECT count(*) FROM issues WHERE parent_id=?1",
                    [k],
                    |r| r.get(0),
                )?;
                if children > 0 {
                    return Err(AppError::validation(
                        "An issue with subtasks cannot become a subtask.",
                    ));
                }
            }
        }
        for label in &input.label_ids {
            let exists = self
                .repo
                .conn
                .query_row(
                    "SELECT 1 FROM labels WHERE id=?1 AND workspace_id=?2",
                    params![label, workspace],
                    |_| Ok(()),
                )
                .optional()?
                .is_some();
            if !exists {
                return Err(AppError::validation("Choose labels from this workspace."));
            }
        }
        let issue_id = key
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let now = chrono::Utc::now().to_rfc3339();
        let mut fields = Vec::new();
        if let Some(old) = &old {
            if old.title != input.title {
                fields.push("title");
            }
            if old.description != input.description {
                fields.push("description");
            }
            if old.status != input.status {
                fields.push("status");
            }
            if old.priority != input.priority {
                fields.push("priority");
            }
            if old.due_date != input.due_date {
                fields.push("due date");
            }
            if old.parent_id != input.parent_id {
                fields.push("parent");
            }
            if old.position != input.position {
                fields.push("order");
            }
            let mut before = old.label_ids.clone();
            before.sort();
            let mut after = input.label_ids.clone();
            after.sort();
            if before != after {
                fields.push("labels");
            }
            if fields.is_empty() {
                return Ok(old.clone());
            }
        }
        let tx = self.repo.conn.transaction()?;
        if key.is_some() {
            changed(tx.execute("UPDATE issues SET title=?1,description=?2,status=?3,priority=?4,due_date=?5,parent_id=?6,position=?7,updated_at=?8 WHERE id=?9 AND project_id=?10", params![input.title,input.description,input.status,input.priority,input.due_date,input.parent_id,input.position,now,issue_id,input.project_id])?)?;
        } else {
            tx.execute(
                "UPDATE projects SET issue_sequence=issue_sequence+1 WHERE id=?1",
                [&input.project_id],
            )?;
            let number: i64 = tx.query_row(
                "SELECT issue_sequence FROM projects WHERE id=?1",
                [&input.project_id],
                |r| r.get(0),
            )?;
            tx.execute("INSERT INTO issues(id,project_id,number,title,description,status,priority,due_date,parent_id,position,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?11)",params![issue_id,input.project_id,number,input.title,input.description,input.status,input.priority,input.due_date,input.parent_id,input.position,now])?;
        }
        tx.execute("DELETE FROM issue_labels WHERE issue_id=?1", [&issue_id])?;
        for label in &input.label_ids {
            tx.execute(
                "INSERT INTO issue_labels(issue_id,label_id,created_at) VALUES(?1,?2,?3)",
                params![issue_id, label, now],
            )?;
        }
        issue_event(
            &tx,
            &workspace,
            &input.project_id,
            &issue_id,
            if key.is_some() {
                "issue.updated"
            } else {
                "issue.created"
            },
            &if key.is_some() {
                format!("Changed {}", fields.join(", "))
            } else {
                "Issue created".into()
            },
            &now,
        )?;
        tx.commit()?;
        self.issue(&input.project_id, &issue_id)
    }
    pub fn archive_issue(&mut self, project: &str, key: &str, archived: bool) -> Result<()> {
        let issue = self.issue(project, key)?;
        if issue.archived == archived {
            return Ok(());
        }
        let workspace = self.repo.project_workspace(project)?;
        let now = chrono::Utc::now().to_rfc3339();
        let tx = self.repo.conn.transaction()?;
        tx.execute(
            "UPDATE issues SET archived=?1,updated_at=?2 WHERE id=?3",
            params![archived, now, key],
        )?;
        issue_event(
            &tx,
            &workspace,
            project,
            key,
            if archived {
                "issue.archived"
            } else {
                "issue.restored"
            },
            if archived {
                "Issue archived"
            } else {
                "Issue restored"
            },
            &now,
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn delete_issue(&mut self, project: &str, key: &str) -> Result<()> {
        self.agent_require_idle()?;
        self.issue(project, key)?;
        let workspace = self.repo.project_workspace(project)?;
        let now = chrono::Utc::now().to_rfc3339();
        let tx = self.repo.conn.transaction()?;
        // Child issues survive as top-level issues; record the relationship changes atomically.
        let children = tx
            .prepare("SELECT id FROM issues WHERE parent_id=?1")?
            .query_map([key], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        tx.execute(
            "UPDATE issues SET parent_id=NULL,updated_at=?1 WHERE parent_id=?2",
            params![now, key],
        )?;
        for child in children {
            issue_event(
                &tx,
                &workspace,
                project,
                &child,
                "issue.updated",
                "Parent deleted; moved to top level",
                &now,
            )?;
        }
        changed(tx.execute(
            "DELETE FROM issues WHERE id=?1 AND project_id=?2",
            params![key, project],
        )?)?;
        issue_event(
            &tx,
            &workspace,
            project,
            key,
            "issue.deleted",
            "Issue deleted",
            &now,
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn labels(&self, workspace: &str) -> Result<Vec<Label>> {
        id(workspace)?;
        self.repo.workspace_exists(workspace)?;
        self.repo.labels(workspace)
    }
    pub fn save_label(&mut self, key: Option<&str>, mut input: LabelInput) -> Result<Vec<Label>> {
        input.validate()?;
        self.repo.workspace_exists(&input.workspace_id)?;
        if let Some(k) = key {
            id(k)?;
        }
        let duplicate: bool=self.repo.conn.query_row("SELECT EXISTS(SELECT 1 FROM labels WHERE workspace_id=?1 AND lower(trim(name))=lower(?2) AND (?3 IS NULL OR id<>?3))",params![input.workspace_id,input.name,key],|r|r.get(0))?;
        if duplicate {
            return Err(AppError::validation(
                "A label with this name already exists.",
            ));
        }
        let label_id = key
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let now = chrono::Utc::now().to_rfc3339();
        let tx = self.repo.conn.transaction()?;
        if key.is_some() {
            changed(tx.execute(
                "UPDATE labels SET name=?1,color=?2,updated_at=?3 WHERE id=?4 AND workspace_id=?5",
                params![input.name, input.color, now, label_id, input.workspace_id],
            )?)?;
        } else {
            tx.execute("INSERT INTO labels(id,workspace_id,name,color,created_at,updated_at) VALUES(?1,?2,?3,?4,?5,?5)",params![label_id,input.workspace_id,input.name,input.color,now])?;
        }
        crate::repository::event(
            &tx,
            &input.workspace_id,
            None,
            if key.is_some() {
                "label.updated"
            } else {
                "label.created"
            },
            "Label changed",
            &now,
        )?;
        tx.commit()?;
        self.labels(&input.workspace_id)
    }
    pub fn delete_label(&mut self, workspace: &str, key: &str) -> Result<Vec<Label>> {
        id(workspace)?;
        id(key)?;
        self.repo.workspace_exists(workspace)?;
        let now = chrono::Utc::now().to_rfc3339();
        let tx = self.repo.conn.transaction()?;
        let assigned=tx.prepare("SELECT i.id,i.project_id FROM issues i JOIN issue_labels il ON il.issue_id=i.id WHERE il.label_id=?1")?.query_map([key],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<std::result::Result<Vec<_>,_>>()?;
        changed(tx.execute(
            "DELETE FROM labels WHERE id=?1 AND workspace_id=?2",
            params![key, workspace],
        )?)?;
        for (issue, project) in assigned {
            tx.execute(
                "UPDATE issues SET updated_at=?1 WHERE id=?2",
                params![now, issue],
            )?;
            issue_event(
                &tx,
                workspace,
                &project,
                &issue,
                "issue.updated",
                "Label removed after deletion",
                &now,
            )?;
        }
        crate::repository::event(&tx, workspace, None, "label.deleted", "Label deleted", &now)?;
        tx.commit()?;
        self.labels(workspace)
    }
    pub fn issue_activity(
        &self,
        project: &str,
        key: &str,
        offset: i64,
        limit: i64,
    ) -> Result<ActivityPage> {
        self.issue(project, key)?;
        pagination(offset, limit)?;
        self.repo.activity(key, offset, limit)
    }
    pub fn subtasks(&self, project: &str, key: &str, offset: i64, limit: i64) -> Result<IssuePage> {
        self.issue(project, key)?;
        pagination(offset, limit)?;
        self.repo.subtasks(project, key, offset, limit)
    }
}
fn pagination(offset: i64, limit: i64) -> Result<()> {
    if offset < 0 || !(1..=100).contains(&limit) {
        return Err(AppError::validation(
            "Page size must be 1–100 and offset nonnegative.",
        ));
    }
    Ok(())
}
