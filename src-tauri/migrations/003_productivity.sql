-- Presentation state is part of the Rust-owned, backed-up application state.
CREATE TABLE project_views (
 project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
 view TEXT NOT NULL CHECK(view IN ('list','board'))
);
CREATE TABLE backup_metadata (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 format_version INTEGER NOT NULL,
 created_at TEXT NOT NULL
);

ALTER TABLE app_state ADD COLUMN theme TEXT CHECK(theme IN ('light','dark','system'));

-- Measured board sorting used a temporary B-tree at 10k rows.
CREATE INDEX issues_board_order ON issues(project_id,archived,status,position,number,id);
