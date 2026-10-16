//! Informational discovery is never executable authorization.
use crate::{agent_provider as engine, domain::*, service::Service};
use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    sync::{atomic::AtomicBool, Arc},
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub token: String,
    pub path: String,
    pub sha256: String,
}
#[derive(Clone)]
pub(crate) struct Discovered {
    pub provider: String,
    pub candidate: Candidate,
    pub created: i64,
}
pub(crate) struct Login {
    pub cancel: Arc<AtomicBool>,
    pub state: Arc<std::sync::Mutex<String>>,
}
impl Drop for Login {
    fn drop(&mut self) {
        self.cancel.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Onboarding {
    pub provider: String,
    pub platform: String,
    pub candidates: Vec<Candidate>,
    pub authorized: bool,
    pub compatible: bool,
    pub authentication: String,
    pub ready: bool,
    pub diagnostics: String,
    pub prerequisites: Vec<String>,
    pub login: String,
}
// Explicit platform input makes every platform's search policy testable on any host.
pub(crate) fn locations(
    platform: &str,
    provider: &str,
    home: Option<&Path>,
    local_app_data: Option<&Path>,
    path: Vec<PathBuf>,
) -> Vec<PathBuf> {
    let name = if platform == "windows" {
        format!("{provider}.exe")
    } else {
        provider.into()
    };
    let mut dirs = path;
    if let Some(home) = home {
        dirs.push(home.join(".local/bin"));
    }
    if platform == "windows" && provider == "codex" {
        if let Some(local) = local_app_data {
            dirs.push(local.join("Programs/OpenAI/Codex/bin"));
        }
    }
    if platform == "macos" {
        dirs.extend([
            PathBuf::from("/opt/homebrew/bin"),
            PathBuf::from("/usr/local/bin"),
        ]);
    }
    let mut result = Vec::new();
    for dir in dirs.into_iter().filter(|p| p.is_absolute()).take(128) {
        let file = dir.join(&name);
        if !result.contains(&file) {
            result.push(file);
        }
    }
    result
}
fn paths() -> Vec<PathBuf> {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default()
}
fn discover(provider: &str) -> Vec<Candidate> {
    let home =
        std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from);
    let local = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let mut dirs = paths();
    if provider == "codex" {
        if let Some(dir) = std::env::var_os("CODEX_INSTALL_DIR") {
            dirs.push(PathBuf::from(dir));
        }
    }
    scan_files(locations(
        std::env::consts::OS,
        provider,
        home.as_deref(),
        local.as_deref(),
        dirs,
    ))
}
pub(crate) fn scan_files(files: Vec<PathBuf>) -> Vec<Candidate> {
    let mut candidates: Vec<Candidate> = Vec::new();
    for path in files.into_iter().take(128) {
        if let Ok(exe) = engine::executable(&path) {
            let path = exe.to_string_lossy().into_owned();
            if candidates.iter().any(|c| c.path == path) {
                continue;
            }
            if let Ok(sha256) = engine::digest(&exe) {
                candidates.push(Candidate {
                    token: uuid::Uuid::new_v4().to_string(),
                    path,
                    sha256,
                });
            }
        }
        if candidates.len() == 16 {
            break;
        }
    }
    candidates
}
pub(crate) fn auth_result(provider: &str, result: &engine::Execution) -> &'static str {
    if result.status != "Completed" && result.exit != Some(1) {
        return "unknown";
    }
    if provider == "claude" {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(result.log.trim()) {
            return match (result.exit, value.get("loggedIn").and_then(|v| v.as_bool())) {
                (Some(0), Some(true)) => "authenticated",
                (Some(1), Some(false)) => "unauthenticated",
                _ => "unknown",
            };
        }
    } else if result.exit == Some(0) && result.log.to_lowercase().contains("logged in") {
        return "authenticated";
    } else if result.exit == Some(1) && result.log.to_lowercase().contains("not logged in") {
        return "unauthenticated";
    }
    "unknown"
}
impl Service {
    pub fn agent_onboarding(&mut self, provider: &str) -> Result<Onboarding> {
        engine::provider(provider)?;
        self.agents.discovered.retain(|_, c| {
            chrono::Utc::now().timestamp() - c.created < 300 && c.provider != provider
        });
        let candidates = discover(provider);
        for c in &candidates {
            self.agents.discovered.insert(
                c.token.clone(),
                Discovered {
                    provider: provider.into(),
                    candidate: c.clone(),
                    created: chrono::Utc::now().timestamp(),
                },
            );
        }
        let mut result = Onboarding {
            provider: provider.into(),
            platform: std::env::consts::OS.into(),
            candidates,
            authorized: false,
            compatible: false,
            authentication: "unavailable".into(),
            ready: false,
            diagnostics: String::new(),
            prerequisites: Vec::new(),
            login: self
                .agents
                .login
                .as_ref()
                .map(|l| l.state.lock().unwrap_or_else(|e| e.into_inner()).clone())
                .unwrap_or_else(|| "idle".into()),
        };
        for name in ["git", "node", "npm"] {
            let present = paths().into_iter().filter(|p| p.is_absolute()).any(|p| {
                p.join(if cfg!(windows) {
                    format!("{name}.exe")
                } else {
                    name.into()
                })
                .is_file()
                    || (cfg!(windows) && p.join(format!("{name}.cmd")).is_file())
            });
            if !present {
                result.prerequisites.push(name.into());
            }
        }
        match self.agent_executable(provider) {
            Err(e) => result.diagnostics = e.message,
            Ok(exe) => {
                result.authorized = true;
                match engine::probe(&exe, provider) {
                    Err(e) => result.diagnostics = e.message,
                    Ok(_) => {
                        result.compatible = true;
                        let args = if provider == "codex" {
                            vec!["login".into(), "status".into()]
                        } else {
                            vec!["auth".into(), "status".into()]
                        };
                        // Recheck after capability inspection as well as before it.
                        let exe = self.agent_executable(provider)?;
                        result.authentication = match engine::execute(
                            &exe,
                            &std::env::temp_dir(),
                            &args,
                            "",
                            engine::Limits {
                                runtime: 5,
                                output_bytes: 16384,
                            },
                            Arc::new(AtomicBool::new(false)),
                            |_| {},
                        ) {
                            Ok(status) => auth_result(provider, &status).into(),
                            Err(_) => "unavailable".into(),
                        };
                        let settings = self
                            .agent_settings()?
                            .into_iter()
                            .find(|s| s.provider == provider)
                            .ok_or_else(AppError::missing)?;
                        result.ready = result.authentication == "authenticated"
                            && !settings.require_os_isolation
                            && engine::arguments(provider, &settings.policy).is_ok();
                        if !result.ready && result.authentication == "authenticated" {
                            result.diagnostics = "Permission restrictions prevent execution. Review Agent settings; whole-provider OS isolation is unavailable. No policy was changed.".into();
                        }
                    }
                }
            }
        }
        Ok(result)
    }
    pub fn agent_authorize_discovered(&mut self, token: &str, confirmed: bool) -> Result<()> {
        if !confirmed {
            return Err(engine::error(
                "Explicit executable authorization is required.",
            ));
        }
        let c = self
            .agents
            .discovered
            .remove(token)
            .ok_or_else(|| engine::error("Discovery expired. Rescan and authorize again."))?;
        if chrono::Utc::now().timestamp() - c.created >= 300 {
            return Err(engine::error(
                "Discovery expired. Rescan and authorize again.",
            ));
        }
        let path = engine::executable(Path::new(&c.candidate.path))?;
        if path.to_str() != Some(c.candidate.path.as_str())
            || engine::digest(&path)? != c.candidate.sha256
        {
            return Err(engine::error(
                "Executable changed since discovery. Rescan and review its identity again.",
            ));
        }
        self.agent_authorize_identity(&c.provider, &path, Some(&c.candidate.sha256))
    }
    pub fn agent_login(&mut self, provider: &str, approved: bool) -> Result<()> {
        if !approved {
            return Err(engine::error(
                "Explicit approval is required to open provider login.",
            ));
        }
        engine::provider(provider)?;
        if self
            .agents
            .login
            .as_ref()
            .is_some_and(|l| *l.state.lock().unwrap_or_else(|e| e.into_inner()) == "running")
        {
            return Err(engine::error(
                "A login is already running. Finish or cancel it first.",
            ));
        }
        let exe = self.agent_executable(provider)?;
        engine::probe(&exe, provider)?;
        let exe = self.agent_executable(provider)?;
        let digest = engine::digest(&exe)?;
        let args = if provider == "codex" {
            vec!["login".into()]
        } else {
            vec!["auth".into(), "login".into()]
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let state = Arc::new(std::sync::Mutex::new("running".to_string()));
        let worker_state = state.clone();
        let worker_cancel = cancel.clone();
        std::thread::Builder::new()
            .name("provider-login".into())
            .spawn(move || {
                let result = (|| -> Result<engine::Execution> {
                    if engine::executable(&exe)? != exe || engine::digest(&exe)? != digest {
                        return Err(engine::error("Executable changed."));
                    }
                    engine::execute(
                        &exe,
                        &std::env::temp_dir(),
                        &args,
                        "",
                        engine::Limits {
                            runtime: 180,
                            output_bytes: 16384,
                        },
                        worker_cancel,
                        |_| {},
                    )
                })();
                // Never retain provider login output, URLs, device codes or credentials.
                *worker_state.lock().unwrap_or_else(|e| e.into_inner()) = match result {
                    Ok(r) if r.status == "Completed" => "finished",
                    Ok(r) if r.status == "Cancelled" => "cancelled",
                    _ => "failed",
                }
                .into();
            })
            .map_err(|_| {
                engine::error(
                    "Could not start login. Use the official CLI login command in a terminal.",
                )
            })?;
        self.agents.login = Some(Login { cancel, state });
        Ok(())
    }
    pub fn agent_cancel_login(&mut self) -> Result<()> {
        if let Some(login) = &self.agents.login {
            login
                .cancel
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    }
}
