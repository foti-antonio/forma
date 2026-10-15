-- Append-only upgrade: existing issue rows and label assignments retain their IDs.
ALTER TABLE projects ADD COLUMN issue_sequence INTEGER NOT NULL DEFAULT 0 CHECK(issue_sequence >= 0);
CREATE TABLE project_keys (
 number INTEGER PRIMARY KEY AUTOINCREMENT,
 project_id TEXT NOT NULL UNIQUE REFERENCES projects(id) ON DELETE CASCADE
);
INSERT INTO project_keys(project_id) SELECT id FROM projects ORDER BY created_at,id;
CREATE TRIGGER project_key_created AFTER INSERT ON projects BEGIN
 INSERT INTO project_keys(project_id) VALUES(NEW.id);
END;
ALTER TABLE issues ADD COLUMN number INTEGER NOT NULL DEFAULT 0 CHECK(number >= 0);
ALTER TABLE issues ADD COLUMN priority TEXT NOT NULL DEFAULT 'none' CHECK(priority IN ('urgent','high','medium','low','none'));
ALTER TABLE issues ADD COLUMN position INTEGER NOT NULL DEFAULT 0 CHECK(position >= 0);
ALTER TABLE issues ADD COLUMN due_date TEXT CHECK(due_date IS NULL OR (length(due_date)=10 AND due_date GLOB '[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]'));
ALTER TABLE issues ADD COLUMN archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0,1));
ALTER TABLE issues ADD COLUMN parent_id TEXT REFERENCES issues(id) ON DELETE SET NULL;
UPDATE issues SET number=(SELECT count(*) FROM issues other WHERE other.project_id=issues.project_id AND (other.created_at < issues.created_at OR (other.created_at=issues.created_at AND other.id <= issues.id)));
UPDATE issues SET position=number;
UPDATE projects SET issue_sequence=COALESCE((SELECT max(number) FROM issues WHERE project_id=projects.id),0);
CREATE UNIQUE INDEX issues_number ON issues(project_id,number);
CREATE INDEX issues_list ON issues(project_id,archived,position,id);
CREATE INDEX issues_status ON issues(project_id,archived,status,priority);
CREATE INDEX issues_parent ON issues(parent_id);
-- M01 had no public issue writes, but preserve legacy rows and normalize statuses.
UPDATE issues SET status='backlog' WHERE status NOT IN ('backlog','todo','in_progress','done','cancelled');
CREATE TRIGGER issue_validate_insert BEFORE INSERT ON issues BEGIN
 SELECT RAISE(ABORT,'Invalid issue') WHERE length(trim(NEW.title)) NOT BETWEEN 1 AND 200 OR length(NEW.description)>50000 OR NEW.number<1 OR NEW.status NOT IN ('backlog','todo','in_progress','done','cancelled');
 SELECT RAISE(ABORT,'Invalid parent') WHERE NEW.parent_id IS NOT NULL AND (NEW.parent_id=NEW.id OR NOT EXISTS(SELECT 1 FROM issues p WHERE p.id=NEW.parent_id AND p.project_id=NEW.project_id AND p.parent_id IS NULL));
END;
CREATE TRIGGER issue_validate_update BEFORE UPDATE ON issues BEGIN
 SELECT RAISE(ABORT,'Invalid issue') WHERE length(trim(NEW.title)) NOT BETWEEN 1 AND 200 OR length(NEW.description)>50000 OR NEW.number<1 OR NEW.status NOT IN ('backlog','todo','in_progress','done','cancelled');
 SELECT RAISE(ABORT,'Immutable identity') WHERE NEW.project_id<>OLD.project_id OR NEW.number<>OLD.number;
 SELECT RAISE(ABORT,'Invalid parent') WHERE NEW.parent_id IS NOT NULL AND (NEW.parent_id=NEW.id OR EXISTS(SELECT 1 FROM issues c WHERE c.parent_id=NEW.id) OR NOT EXISTS(SELECT 1 FROM issues p WHERE p.id=NEW.parent_id AND p.project_id=NEW.project_id AND p.parent_id IS NULL));
END;
CREATE INDEX labels_name_folded ON labels(workspace_id,lower(trim(name)));
CREATE TRIGGER issue_label_workspace BEFORE INSERT ON issue_labels BEGIN
 SELECT RAISE(ABORT,'Cross-workspace label') WHERE NOT EXISTS(SELECT 1 FROM issues i JOIN projects p ON p.id=i.project_id JOIN labels l ON l.workspace_id=p.workspace_id WHERE i.id=NEW.issue_id AND l.id=NEW.label_id);
END;
ALTER TABLE activity_events ADD COLUMN issue_id TEXT;
-- Retain the identifier after deletion, without retaining issue content.
CREATE INDEX activity_issue ON activity_events(issue_id,created_at,id);
