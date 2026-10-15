CREATE TABLE agent_settings (
 provider TEXT PRIMARY KEY CHECK(provider IN ('codex','claude')),
 executable TEXT,
 executable_digest TEXT,
 trusted INTEGER NOT NULL DEFAULT 0 CHECK(trusted IN (0,1)),
 max_runtime INTEGER NOT NULL DEFAULT 900 CHECK(max_runtime BETWEEN 1 AND 7200),
 log_limit INTEGER NOT NULL DEFAULT 65536 CHECK(log_limit BETWEEN 1024 AND 1048576),
 policy TEXT NOT NULL DEFAULT 'restricted' CHECK(policy IN ('restricted','claude-tools'))
);
INSERT INTO agent_settings(provider) VALUES('codex'),('claude');
UPDATE agent_settings SET policy='claude-tools' WHERE provider='claude';
CREATE TABLE agent_preferences (
 singleton INTEGER PRIMARY KEY CHECK(singleton=1),
 preferred TEXT NOT NULL DEFAULT 'codex' CHECK(preferred IN ('codex','claude'))
);
INSERT INTO agent_preferences(singleton) VALUES(1);
CREATE TABLE agent_runs (
 id TEXT PRIMARY KEY,
 issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
 repository_id TEXT NOT NULL REFERENCES git_repositories(id) ON DELETE CASCADE,
 provider TEXT NOT NULL CHECK(provider IN ('codex','claude')),
 worktree TEXT NOT NULL,
 branch TEXT NOT NULL UNIQUE,
 base_commit TEXT NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('Queued','Running','Completed','Failed','Cancelled','Interrupted')),
 started_at TEXT NOT NULL,
 ended_at TEXT,
 exit_code INTEGER,
 result TEXT NOT NULL DEFAULT '' CHECK(length(result)<=4000),
 log TEXT NOT NULL DEFAULT '' CHECK(length(CAST(log AS BLOB))<=1048576),
 review TEXT CHECK(review IN ('accepted','rejected')),
 trusted INTEGER NOT NULL DEFAULT 1 CHECK(trusted IN (0,1))
);
CREATE UNIQUE INDEX agent_active_issue ON agent_runs(issue_id) WHERE status IN ('Queued','Running');
CREATE INDEX agent_history ON agent_runs(issue_id,started_at DESC,id);
CREATE TRIGGER agent_run_scope BEFORE INSERT ON agent_runs
WHEN (SELECT project_id FROM issues WHERE id=NEW.issue_id) <> (SELECT project_id FROM git_repositories WHERE id=NEW.repository_id)
BEGIN SELECT RAISE(ABORT,'Agent run repository must belong to the issue project'); END;
CREATE TRIGGER agent_run_scope_update BEFORE UPDATE OF issue_id,repository_id ON agent_runs
WHEN (SELECT project_id FROM issues WHERE id=NEW.issue_id) <> (SELECT project_id FROM git_repositories WHERE id=NEW.repository_id)
BEGIN SELECT RAISE(ABORT,'Agent run repository must belong to the issue project'); END;
