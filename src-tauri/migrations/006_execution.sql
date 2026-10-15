ALTER TABLE agent_preferences ADD COLUMN max_concurrent INTEGER NOT NULL DEFAULT 1 CHECK(max_concurrent IN (1,2));
CREATE TABLE agent_queue (
 id TEXT PRIMARY KEY,
 issue_id TEXT NOT NULL REFERENCES issues(id) ON DELETE CASCADE,
 provider TEXT NOT NULL CHECK(provider IN ('codex','claude')),
 payload TEXT NOT NULL CHECK(length(CAST(payload AS BLOB))<=131072),
 approved_at TEXT NOT NULL,
 expires_at TEXT NOT NULL,
 position INTEGER NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('Approved','Launching','Started','Invalidated','Cancelled')),
 message TEXT NOT NULL DEFAULT '',
 run_id TEXT REFERENCES agent_runs(id) ON DELETE SET NULL
);
CREATE UNIQUE INDEX agent_queue_issue ON agent_queue(issue_id) WHERE status IN ('Approved','Launching');
CREATE INDEX agent_queue_order ON agent_queue(status,position,id);
ALTER TABLE agent_runs ADD COLUMN cleanup_state TEXT NOT NULL DEFAULT 'Retained' CHECK(cleanup_state IN ('Retained','Removing','Removed','Failed'));
ALTER TABLE agent_runs ADD COLUMN cleanup_message TEXT NOT NULL DEFAULT '';
CREATE TABLE project_test_commands (
 project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
 executable TEXT NOT NULL,
 executable_digest TEXT NOT NULL,
 trusted INTEGER NOT NULL DEFAULT 0 CHECK(trusted IN (0,1)),
 arguments TEXT NOT NULL DEFAULT '[]' CHECK(length(CAST(arguments AS BLOB))<=16384),
 max_runtime INTEGER NOT NULL DEFAULT 300 CHECK(max_runtime BETWEEN 1 AND 7200),
 log_limit INTEGER NOT NULL DEFAULT 65536 CHECK(log_limit BETWEEN 1024 AND 1048576)
);
CREATE TABLE independent_tests (
 id TEXT PRIMARY KEY,
 run_id TEXT NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
 executable TEXT NOT NULL,
 arguments TEXT NOT NULL,
 working_directory TEXT NOT NULL,
 status TEXT NOT NULL CHECK(status IN ('Running','Completed','Failed','Cancelled','Interrupted')),
 started_at TEXT NOT NULL,
 ended_at TEXT,
 duration_ms INTEGER,
 exit_code INTEGER,
 result TEXT NOT NULL DEFAULT '',
 log TEXT NOT NULL DEFAULT '' CHECK(length(CAST(log AS BLOB))<=1048576)
);
CREATE UNIQUE INDEX independent_test_active ON independent_tests(run_id) WHERE status='Running';
ALTER TABLE agent_settings ADD COLUMN require_os_isolation INTEGER NOT NULL DEFAULT 0 CHECK(require_os_isolation IN (0,1));
