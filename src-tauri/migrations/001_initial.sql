CREATE TABLE workspaces (
 id TEXT PRIMARY KEY NOT NULL,
 name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 80),
 created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE TABLE projects (
 id TEXT PRIMARY KEY NOT NULL,
 workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
 name TEXT NOT NULL CHECK(length(trim(name)) BETWEEN 1 AND 100),
 description TEXT NOT NULL DEFAULT '' CHECK(length(description) <= 4000),
 icon TEXT NOT NULL DEFAULT 'layers',
 color TEXT NOT NULL DEFAULT '#8b5cf6',
 status TEXT NOT NULL DEFAULT 'planned' CHECK(status IN ('planned','active','paused','completed')),
 archived INTEGER NOT NULL DEFAULT 0 CHECK(archived IN (0,1)),
 created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE INDEX projects_workspace ON projects(workspace_id, archived, updated_at);
CREATE TABLE issues (
 id TEXT PRIMARY KEY NOT NULL, project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
 title TEXT NOT NULL, description TEXT NOT NULL DEFAULT '', status TEXT NOT NULL DEFAULT 'backlog',
 created_at TEXT NOT NULL, updated_at TEXT NOT NULL
);
CREATE INDEX issues_project ON issues(project_id);
CREATE TABLE labels (
 id TEXT PRIMARY KEY NOT NULL, workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
 name TEXT NOT NULL, color TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
 UNIQUE(workspace_id,name)
);
CREATE INDEX labels_workspace ON labels(workspace_id);
CREATE TABLE issue_labels (
 issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
 label_id TEXT NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
 created_at TEXT NOT NULL, PRIMARY KEY(issue_id,label_id)
);
CREATE INDEX issue_labels_label ON issue_labels(label_id);
CREATE TABLE activity_events (
 id TEXT PRIMARY KEY NOT NULL, workspace_id TEXT NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
 project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
 kind TEXT NOT NULL, summary TEXT NOT NULL, created_at TEXT NOT NULL
);
CREATE INDEX activity_workspace ON activity_events(workspace_id,created_at);
CREATE TABLE app_state (
 singleton INTEGER PRIMARY KEY CHECK(singleton = 1),
 active_workspace_id TEXT REFERENCES workspaces(id) ON DELETE SET NULL
);
INSERT INTO app_state(singleton,active_workspace_id) VALUES(1,NULL);
