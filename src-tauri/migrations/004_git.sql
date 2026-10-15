-- One active connection per project; historical repositories and links survive disconnect.
CREATE TABLE git_repositories (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    path TEXT NOT NULL,
    name TEXT NOT NULL,
    connected INTEGER NOT NULL DEFAULT 1 CHECK(connected IN (0,1)),
    trusted INTEGER NOT NULL DEFAULT 1 CHECK(trusted IN (0,1)),
    created_at TEXT NOT NULL,
    UNIQUE(project_id,path),
    UNIQUE(id,project_id)
);
CREATE UNIQUE INDEX git_one_connected ON git_repositories(project_id) WHERE connected=1;
CREATE TABLE git_links (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
    repository_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK(kind IN ('branch','commit')),
    target TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    message TEXT NOT NULL,
    author TEXT NOT NULL,
    committed_at TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY(repository_id,project_id) REFERENCES git_repositories(id,project_id) ON DELETE CASCADE,
    UNIQUE(issue_id,repository_id,kind,target)
);
CREATE INDEX git_links_issue ON git_links(issue_id,created_at,id);
CREATE TRIGGER git_link_scope BEFORE INSERT ON git_links
WHEN NOT EXISTS(SELECT 1 FROM issues WHERE id=NEW.issue_id AND project_id=NEW.project_id)
BEGIN SELECT RAISE(ABORT,'Git link must belong to issue project'); END;
CREATE TRIGGER git_link_scope_update BEFORE UPDATE ON git_links
WHEN NOT EXISTS(SELECT 1 FROM issues WHERE id=NEW.issue_id AND project_id=NEW.project_id)
BEGIN SELECT RAISE(ABORT,'Git link must belong to issue project'); END;
