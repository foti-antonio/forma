//! Independently approved project tests, using an executable and argv without a shell.
use crate::{
    agent_provider as engine, agent_types::*, domain::*, repository::Repository, service::Service,
};
use rusqlite::{params, OptionalExtension};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

impl Service {
    pub fn test_project(&self, run: &str) -> Result<String> {
        id(run)?;
        let record = self.repo.agent_run(run)?;
        Ok(self.repo.conn.query_row(
            "SELECT project_id FROM issues WHERE id=?1",
            [&record.issue_id],
            |r| r.get(0),
        )?)
    }
    pub fn test_command(&self, project: &str) -> Result<Option<TestCommand>> {
        id(project)?;
        let row = self.repo.conn.query_row("SELECT executable,trusted,arguments,max_runtime,log_limit FROM project_test_commands WHERE project_id=?1", [project], |r| Ok((r.get::<_,String>(0)?,r.get(1)?,r.get::<_,String>(2)?,r.get(3)?,r.get(4)?))).optional()?;
        row.map(|(executable, trusted, args, max_runtime, log_limit)| {
            Ok(TestCommand {
                project_id: project.into(),
                executable,
                trusted,
                arguments: serde_json::from_str(&args)
                    .map_err(|_| engine::error("Invalid test arguments. Configure again."))?,
                max_runtime,
                log_limit,
            })
        })
        .transpose()
    }
    // Only a native picker may authorize this path. Selecting it does not execute tests.
    pub fn test_authorize(&self, project: &str, path: &Path) -> Result<()> {
        id(project)?;
        if !self
            .repo
            .conn
            .prepare("SELECT 1 FROM projects WHERE id=?1")?
            .exists([project])?
        {
            return Err(AppError::missing());
        }
        let exe = engine::executable(path)?;
        let name = exe
            .file_stem()
            .and_then(|x| x.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if [
            "sh",
            "bash",
            "zsh",
            "fish",
            "dash",
            "cmd",
            "powershell",
            "pwsh",
            "wscript",
            "cscript",
        ]
        .contains(&name.as_str())
        {
            return Err(engine::error(
                "Select a test runner executable, not a command shell.",
            ));
        }
        self.repo.conn.execute("INSERT INTO project_test_commands(project_id,executable,executable_digest,trusted) VALUES(?1,?2,?3,1) ON CONFLICT(project_id) DO UPDATE SET executable=excluded.executable,executable_digest=excluded.executable_digest,trusted=1", params![project,exe.to_string_lossy(),engine::digest(&exe)?])?;
        Ok(())
    }
    pub fn test_save_command(&self, input: TestCommand) -> Result<()> {
        id(&input.project_id)?;
        if input.arguments.len() > 64
            || input
                .arguments
                .iter()
                .any(|a| a.len() > 1024 || a.contains('\0'))
            || !(1..=7200).contains(&input.max_runtime)
            || !(1024..=1048576).contains(&input.log_limit)
        {
            return Err(AppError::validation("Use at most 64 arguments of 1024 bytes, runtime 1–7200 seconds and output 1024–1048576 bytes."));
        }
        let args = serde_json::to_string(&input.arguments)
            .map_err(|_| engine::error("Invalid arguments."))?;
        if args.len() > 16384 {
            return Err(AppError::validation("Arguments exceed 16 KiB."));
        }
        if self.repo.conn.execute("UPDATE project_test_commands SET arguments=?2,max_runtime=?3,log_limit=?4 WHERE project_id=?1", params![input.project_id,args,input.max_runtime,input.log_limit])? == 0 {return Err(engine::error("Authorize a test executable first."));}
        Ok(())
    }
    pub fn test_preview(&mut self, run: &str) -> Result<TestPreview> {
        id(run)?;
        let record = self.repo.agent_run(run)?;
        if matches!(record.status.as_str(), "Running" | "Queued") {
            return Err(engine::error("Wait for the agent to stop before testing."));
        }
        self.agent_worktree(&record)?;
        let project: String = self.repo.conn.query_row(
            "SELECT project_id FROM issues WHERE id=?1",
            [&record.issue_id],
            |r| r.get(0),
        )?;
        let command = self
            .test_command(&project)?
            .ok_or_else(|| engine::error("Configure a test command first."))?;
        if !command.trusted {
            return Err(engine::error(
                "Reauthorize the test executable using the native picker.",
            ));
        }
        let exe = engine::executable(Path::new(&command.executable))?;
        let digest = engine::digest(&exe)?;
        let expected: String = self.repo.conn.query_row(
            "SELECT executable_digest FROM project_test_commands WHERE project_id=?1",
            [&project],
            |r| r.get(0),
        )?;
        if exe != Path::new(&command.executable) || digest != expected {
            return Err(engine::error("Test executable changed. Select it again."));
        }
        let now = chrono::Utc::now().timestamp();
        self.agents.test_pending.retain(|_, p| p.expires_at > now);
        if self.agents.test_pending.len() >= 16 {
            return Err(engine::error("Too many pending test previews."));
        }
        let preview = TestPreview {
            token: uuid::Uuid::new_v4().to_string(),
            run_id: run.into(),
            command,
            working_directory: record.worktree,
            expires_at: now + 300,
            digest,
        };
        self.agents
            .test_pending
            .insert(preview.token.clone(), preview.clone());
        Ok(preview)
    }
    pub fn test_start(&mut self, token: &str, approved: bool) -> Result<String> {
        if !approved {
            return Err(engine::error("Explicit approval is required to run tests."));
        }
        let preview = self
            .agents
            .test_pending
            .remove(token)
            .ok_or_else(|| engine::error("Test preview expired."))?;
        let current = self.test_preview(&preview.run_id)?;
        self.agents.test_pending.remove(&current.token);
        if preview.expires_at <= chrono::Utc::now().timestamp()
            || preview.digest != current.digest
            || preview.working_directory != current.working_directory
            || serde_json::to_string(&preview.command).ok()
                != serde_json::to_string(&current.command).ok()
        {
            return Err(engine::error(
                "Test command or worktree changed. Preview and approve again.",
            ));
        }
        // Shared process budget, including tests, is at most the configured run limit.
        let running: usize = self.repo.conn.query_row("SELECT (SELECT count(*) FROM agent_runs WHERE status IN ('Queued','Running'))+(SELECT count(*) FROM independent_tests WHERE status='Running')", [], |r|r.get(0))?;
        if running >= self.agent_concurrency()? {
            return Err(engine::error(
                "Execution limit reached. Wait for another run or test to stop.",
            ));
        }
        if self
            .repo
            .conn
            .prepare("SELECT 1 FROM independent_tests WHERE run_id=?1 AND status='Running'")?
            .exists([&preview.run_id])?
        {
            return Err(engine::error(
                "Tests are already running for this worktree.",
            ));
        }
        let db = PathBuf::from(
            self.repo
                .conn
                .path()
                .ok_or_else(|| engine::error("Database unavailable."))?,
        );
        let worker = Repository::open(&db)?;
        let id = uuid::Uuid::new_v4().to_string();
        self.repo.conn.execute("INSERT INTO independent_tests(id,run_id,executable,arguments,working_directory,status,started_at) VALUES(?1,?2,?3,?4,?5,'Running',?6)", params![id,preview.run_id,preview.command.executable,serde_json::to_string(&preview.command.arguments).unwrap(),preview.working_directory,chrono::Utc::now().to_rfc3339()])?;
        let cancel = Arc::new(AtomicBool::new(false));
        self.agent_forget_finished();
        self.agents.active.insert(id.clone(), cancel.clone());
        let worker_id = id.clone();
        let spawn=std::thread::Builder::new().name(format!("test-{id}")).spawn(move || {
            let start=Instant::now();
            let result=(|| {
                let exe=Path::new(&preview.command.executable);
                if engine::executable(exe)? != exe || engine::digest(exe)? != preview.digest {
                    return Err(engine::error("Test executable changed before launch; execution blocked."));
                }
                            engine::execute(Path::new(&preview.command.executable),Path::new(&preview.working_directory),&preview.command.arguments,"",engine::Limits{runtime:preview.command.max_runtime,output_bytes:preview.command.log_limit},cancel,|log| {let _=worker.conn.execute("UPDATE independent_tests SET log=?2 WHERE id=?1", params![worker_id,log]);})
            })();
            let (status,exit,result,log)=match result {Ok(r)=>(r.status,r.exit,if r.status=="Completed" {"Independent test process exited successfully.".into()} else {r.result.replace("Agent", "Independent test process")},r.log),Err(e)=>("Failed",None,e.message,String::new())};
            let _=worker.conn.execute("UPDATE independent_tests SET status=?2,ended_at=?3,duration_ms=?4,exit_code=?5,result=?6,log=?7 WHERE id=?1",params![worker_id,status,chrono::Utc::now().to_rfc3339(),start.elapsed().as_millis() as u64,exit,result,log]);
        });
        if spawn.is_err() {
            self.repo.conn.execute("UPDATE independent_tests SET status='Failed',ended_at=?2,result='Could not start test supervisor.' WHERE id=?1",params![id,chrono::Utc::now().to_rfc3339()])?;
            return Err(engine::error("Could not start test supervisor."));
        }
        Ok(id)
    }
    pub fn test_runs(&self, run: &str) -> Result<Vec<IndependentTest>> {
        id(run)?;
        Ok(self.repo.conn.prepare("SELECT id,run_id,executable,arguments,working_directory,status,started_at,ended_at,duration_ms,exit_code,result,log FROM independent_tests WHERE run_id=?1 ORDER BY started_at DESC,id LIMIT 20")?.query_map([run], |r| Ok(IndependentTest {id:r.get(0)?,run_id:r.get(1)?,executable:r.get(2)?,arguments:serde_json::from_str(&r.get::<_,String>(3)?).unwrap_or_default(),working_directory:r.get(4)?,status:r.get(5)?,started_at:r.get(6)?,ended_at:r.get(7)?,duration_ms:r.get(8)?,exit_code:r.get(9)?,result:r.get(10)?,log:r.get(11)?}))?.collect::<std::result::Result<_,_>>()?)
    }
    pub fn test_cancel(&self, test: &str) -> Result<()> {
        id(test)?;
        if self
            .repo
            .conn
            .prepare("SELECT 1 FROM independent_tests WHERE id=?1 AND status='Running'")?
            .exists([test])?
        {
            self.agents
                .active
                .get(test)
                .ok_or_else(|| engine::error("Test supervisor unavailable. Restart to recover."))?
                .store(true, Ordering::SeqCst);
        }
        Ok(())
    }
}
