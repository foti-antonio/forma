//! Provider adapters never accept frontend arguments or invoke a shell.
use crate::domain::*;
use command_group::CommandGroup;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc,
    },
    time::{Duration, Instant},
};

pub(crate) fn error(message: &str) -> AppError {
    AppError {
        code: "agent",
        message: message.into(),
    }
}
pub(crate) fn provider(value: &str) -> Result<()> {
    if !["codex", "claude"].contains(&value) {
        return Err(AppError::validation("Choose Codex or Claude Code."));
    }
    Ok(())
}
pub(crate) fn executable(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute()
        || path
            .as_os_str()
            .to_string_lossy()
            .chars()
            .any(char::is_control)
    {
        return Err(error(
            "Choose an absolute executable path using the native picker.",
        ));
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| error("Executable is missing or inaccessible. Select it again."))?;
    if !canonical.is_file() {
        return Err(error("Select an executable file."));
    }
    #[cfg(windows)]
    if canonical
        .extension()
        .and_then(|x| x.to_str())
        .is_none_or(|x| !x.eq_ignore_ascii_case("exe"))
    {
        return Err(error(
            "Choose a native .exe installation; shell wrappers (.cmd/.bat) are not supported.",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = canonical
            .metadata()
            .map_err(|_| error("Cannot inspect executable."))?
            .permissions()
            .mode();
        if mode & 0o111 == 0 || mode & 0o022 != 0 {
            return Err(error(
                "Executable must be executable and must not be group/world writable.",
            ));
        }
    }
    Ok(canonical)
}
pub(crate) fn digest(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path).map_err(|_| error("Cannot read executable identity."))?;
    let size = file
        .metadata()
        .map_err(|_| error("Cannot inspect executable identity."))?
        .len();
    if size > 512 * 1024 * 1024 {
        return Err(error("Executable exceeds the 512 MiB inspection limit."));
    }
    let mut h = Sha256::new();
    let mut reader = file;
    let mut buffer = [0u8; 16384];
    loop {
        let n = reader
            .read(&mut buffer)
            .map_err(|_| error("Cannot read executable identity."))?;
        if n == 0 {
            break;
        }
        h.update(&buffer[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
pub(crate) fn discover(name: &str) -> Option<String> {
    let filename = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.into()
    };
    std::env::split_paths(&std::env::var_os("PATH")?)
        .filter(|p| p.is_absolute())
        .find_map(|p| {
            executable(&p.join(&filename))
                .ok()
                .map(|p| p.to_string_lossy().into_owned())
        })
}
fn command(exe: &Path, cwd: &Path, args: &[String]) -> Command {
    let mut c = Command::new(exe);
    c.args(args).current_dir(cwd).env_clear();
    // Provider-owned auth files remain usable. No inherited API keys, tokens,
    // loader injection or arbitrary application environment is forwarded.
    for key in [
        "HOME",
        "USERPROFILE",
        "APPDATA",
        "LOCALAPPDATA",
        "SystemRoot",
        "WINDIR",
        "TEMP",
        "TMP",
        "TMPDIR",
        "PATH",
        "LANG",
        "LC_ALL",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
    ] {
        if let Some(v) = std::env::var_os(key) {
            c.env(key, v);
        }
    }
    c.env("NO_COLOR", "1").env("GIT_TERMINAL_PROMPT", "0");
    c
}
// Read whole lines up to 8 KiB. Overlong lines are dropped, not split across
// redaction boundaries. Bounded channel supplies backpressure while draining.
fn reader(
    mut r: impl Read + Send + 'static,
    tx: mpsc::SyncSender<String>,
    total: Arc<AtomicUsize>,
) {
    std::thread::spawn(move || {
        let mut line = Vec::with_capacity(8192);
        let mut overflow = false;
        let mut private_key = false;
        let mut b = [0u8; 1024];
        loop {
            let n = match r.read(&mut b) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            total.fetch_add(n, Ordering::Relaxed);
            for v in &b[..n] {
                if *v == b'\n' {
                    let value = if overflow {
                        "[overlong output line omitted]".into()
                    } else {
                        redact_line(&String::from_utf8_lossy(&line), &mut private_key)
                    };
                    if tx.send(value).is_err() {
                        return;
                    }
                    line.clear();
                    overflow = false;
                } else if line.len() < 8192 {
                    line.push(*v);
                } else {
                    overflow = true;
                }
            }
        }
        if !line.is_empty() {
            let _ = tx.send(if overflow {
                "[overlong output line omitted]".into()
            } else {
                redact_line(&String::from_utf8_lossy(&line), &mut private_key)
            });
        }
    });
}
fn redact_line(value: &str, private_key: &mut bool) -> String {
    if value.contains("-----BEGIN") && value.contains("PRIVATE KEY-----") {
        *private_key = true;
    }
    if *private_key {
        if value.contains("-----END") && value.contains("PRIVATE KEY-----") {
            *private_key = false;
        }
        return "[private key data omitted]".into();
    }
    redact(value)
}
pub(crate) fn redact_block(value: &str) -> String {
    let mut private = false;
    value
        .lines()
        .map(|line| redact_line(line, &mut private))
        .collect::<Vec<_>>()
        .join("\n")
}
pub(crate) fn redact(value: &str) -> String {
    use std::sync::OnceLock;
    static SECRET: OnceLock<regex::Regex> = OnceLock::new();
    let pattern=SECRET.get_or_init(||regex::Regex::new(r#"(?i)(?:\bsk-[a-z0-9_-]{8,}|\bgh[pousr]_[a-z0-9_]{8,}|(?:api[_-]?key|access[_-]?token|authorization|password|secret|bearer)[\s\"':=]+[^\s,\"}]+|-----BEGIN[^\n]*PRIVATE KEY-----.*)"#).expect("static secret pattern"));
    let clean: String = value
        .chars()
        .filter(|c| !c.is_control() || *c == '\t')
        .collect();
    pattern.replace_all(&clean, "[REDACTED]").into_owned()
}
fn append_log(log: &mut String, line: &str, limit: usize, truncated: &mut bool) {
    if *truncated {
        return;
    }
    if log.len() + line.len() < limit {
        log.push_str(line);
        log.push('\n');
    } else {
        let marker = "\n[log limit reached; further output discarded]\n";
        while log.len() + marker.len() > limit {
            log.pop();
        }
        log.push_str(marker);
        *truncated = true;
    }
}
pub(crate) struct Execution {
    pub status: &'static str,
    pub exit: Option<i32>,
    pub result: &'static str,
    pub log: String,
}
pub(crate) struct Limits {
    pub runtime: u64,
    pub output_bytes: usize,
}
pub(crate) fn execute(
    exe: &Path,
    cwd: &Path,
    args: &[String],
    prompt: &str,
    limits: Limits,
    cancel: Arc<AtomicBool>,
    mut progress: impl FnMut(&str),
) -> Result<Execution> {
    let Limits {
        runtime,
        output_bytes: limit,
    } = limits;
    if cancel.load(Ordering::SeqCst) {
        return Ok(Execution {
            status: "Cancelled",
            exit: None,
            result: "Execution cancelled before launch.",
            log: String::new(),
        });
    }
    let mut cmd = command(exe, cwd, args);
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut group = cmd.group();
    #[cfg(windows)]
    group.kill_on_drop(true);
    let child = group.spawn().map_err(|_| {
        error("Agent could not start. Check executable permissions and installation.")
    })?;
    let mut child = ProcessGuard(child, false);
    #[cfg(unix)]
    let mut watchdog = Watchdog::start(child.0.id())?;
    let (tx, rx) = mpsc::sync_channel(32);
    let total = Arc::new(AtomicUsize::new(0));
    if let Some(out) = child.0.inner().stdout.take() {
        reader(out, tx.clone(), total.clone());
    }
    if let Some(out) = child.0.inner().stderr.take() {
        reader(out, tx.clone(), total.clone());
    }
    drop(tx);
    let stdin = child.0.inner().stdin.take();
    let input = prompt.as_bytes().to_vec();
    std::thread::spawn(move || {
        if let Some(mut stdin) = stdin {
            let _ = stdin.write_all(&input);
        }
    });
    let start = Instant::now();
    let mut log = String::new();
    let mut truncated = false;
    let mut last = Instant::now();
    let mut exit = None;
    let mut terminal = None;
    loop {
        // Bound each drain so flood output cannot starve cancellation or timeout.
        for _ in 0..64 {
            let Ok(line) = rx.try_recv() else { break };
            append_log(&mut log, &line, limit, &mut truncated);
        }
        if last.elapsed() >= Duration::from_millis(200) {
            progress(&log);
            last = Instant::now();
        }
        if cancel.load(Ordering::SeqCst) {
            terminal = Some(("Cancelled", "Execution cancelled; worktree retained."));
            break;
        }
        if truncated || total.load(Ordering::Relaxed) > limit {
            terminal = Some((
                "Failed",
                "Output limit reached; process group terminated. Worktree retained.",
            ));
            break;
        }
        if start.elapsed() >= Duration::from_secs(runtime) {
            terminal = Some((
                "Failed",
                "Maximum runtime exceeded; process group terminated.",
            ));
            break;
        }
        match child.0.inner().try_wait() {
            Ok(Some(code)) => {
                exit = code.code();
                break;
            }
            Ok(None) => {}
            Err(_) => {
                terminal = Some((
                    "Failed",
                    "Unable to observe execution; process group terminated.",
                ));
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    // Always kill surviving descendants even after a normal leader exit.
    let _ = child.0.kill();
    let _ = child.0.wait();
    child.1 = true;
    #[cfg(unix)]
    watchdog.finish();
    let drain_deadline = Instant::now() + Duration::from_millis(250);
    for _ in 0..8192 {
        if Instant::now() >= drain_deadline {
            break;
        }
        match rx.recv_timeout(Duration::from_millis(10)) {
            Ok(line) => append_log(&mut log, &line, limit, &mut truncated),
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }
    }
    progress(&log);
    if terminal.is_none() && (truncated || total.load(Ordering::Relaxed) > limit) {
        terminal = Some((
            "Failed",
            "Output limit reached; process group terminated. Worktree retained.",
        ));
    }
    let (status,result)=terminal.unwrap_or(if exit==Some(0) {("Completed","Agent exited successfully. Test results in logs are untrusted provider reports; review the diff.")} else {("Failed","Agent exited unsuccessfully. Inspect logs and the retained worktree.")});
    Ok(Execution {
        status,
        exit,
        result,
        log,
    })
}
pub(crate) fn probe(exe: &Path, provider_name: &str) -> Result<(String, String, String)> {
    provider(provider_name)?;
    let cwd = std::env::temp_dir();
    let run = |args: &[&str]| -> Result<String> {
        let result = execute(
            exe,
            &cwd,
            &args.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            "",
            Limits {
                runtime: 5,
                output_bytes: 65536,
            },
            Arc::new(AtomicBool::new(false)),
            |_| {},
        )?;
        if result.exit != Some(0) {
            return Err(error("Provider capability check failed or timed out."));
        }
        Ok(result.log)
    };
    let version = run(&["--version"])?;
    let help = run(&["--help"])?;
    let exec = if provider_name == "codex" {
        run(&["exec", "--help"])?
    } else {
        help.clone()
    };
    let required = adapter(provider_name)?.required_flags();
    let mut missing = required
        .iter()
        .filter(|flag| !exec.contains(**flag))
        .copied()
        .collect::<Vec<_>>();
    if provider_name == "codex" {
        for flag in ["--ask-for-approval", "--no-daemon"] {
            if !help.contains(flag) {
                missing.push(flag);
            }
        }
    }
    if !missing.is_empty() {
        return Err(error(&format!(
            "Installed CLI lacks required security options: {}. Install a supported version.",
            missing.join(", ")
        )));
    }
    let prefix = if provider_name == "codex" {
        "codex-cli "
    } else {
        ""
    };
    if !version.starts_with(prefix)
        || (provider_name == "claude" && !version.contains("Claude Code"))
    {
        return Err(error(
            "Executable did not identify as the selected provider.",
        ));
    }
    Ok((version.trim().chars().take(200).collect(), help, exec))
}
trait CliProvider {
    fn policy(&self) -> &'static str;
    fn required_flags(&self) -> &'static [&'static str];
    fn args(&self) -> &'static [&'static str];
}
struct Codex;
struct Claude;
impl CliProvider for Codex {
    fn policy(&self) -> &'static str {
        "restricted"
    }
    fn required_flags(&self) -> &'static [&'static str] {
        &[
            "--sandbox",
            "--json",
            "--ignore-user-config",
            "--ignore-rules",
            "--ephemeral",
            "--color",
        ]
    }
    fn args(&self) -> &'static [&'static str] {
        &[
            "--no-daemon",
            "--ask-for-approval",
            "never",
            "exec",
            "--ignore-user-config",
            "--ignore-rules",
            "--ephemeral",
            "--sandbox",
            "workspace-write",
            "--json",
            "--color",
            "never",
            "-c",
            "sandbox_workspace_write.network_access=false",
            "-c",
            "sandbox_workspace_write.exclude_tmpdir_env_var=true",
            "-c",
            "sandbox_workspace_write.exclude_slash_tmp=true",
            "-",
        ]
    }
}
impl CliProvider for Claude {
    fn policy(&self) -> &'static str {
        "claude-tools"
    }
    fn required_flags(&self) -> &'static [&'static str] {
        &[
            "--restricted",
            "--print",
            "--output-format",
            "--permission-mode",
            "--tools",
            "--allowedTools",
            "--no-session-persistence",
            "--strict-mcp-config",
            "--mcp-config",
            "--settings",
            "--verbose",
            "dontAsk",
            "stream-json",
        ]
    }
    fn args(&self) -> &'static [&'static str] {
        &[
            "--restricted",
            "--print",
            "--output-format",
            "stream-json",
            "--verbose",
            "--no-session-persistence",
            "--permission-mode",
            "dontAsk",
            "--tools",
            "Read,Edit,Write,Glob,Grep",
            "--allowedTools",
            "Read,Edit,Write,Glob,Grep",
            "--strict-mcp-config",
            "--mcp-config",
            "{\"mcpServers\":{}}",
            "--settings",
            "{\"disableAllHooks\":true}",
        ]
    }
}
fn adapter(provider: &str) -> Result<&'static dyn CliProvider> {
    match provider {
        "codex" => Ok(&Codex),
        "claude" => Ok(&Claude),
        _ => Err(error("Unsupported provider.")),
    }
}
pub(crate) fn arguments(provider: &str, policy: &str) -> Result<Vec<String>> {
    let adapter = adapter(provider)?;
    if adapter.policy() != policy {
        return Err(error("Codex requires restricted policy. Claude requires Claude tools policy and separate per-run risk approval."));
    }
    Ok(adapter.args().iter().map(|s| s.to_string()).collect())
}

// RAII also covers supervisor panic/unwinding.
struct ProcessGuard(command_group::GroupChild, bool);
impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if !self.1 {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}
#[cfg(unix)]
struct Watchdog {
    child: std::process::Child,
    input: Option<std::process::ChildStdin>,
}
#[cfg(unix)]
impl Watchdog {
    fn start(group: u32) -> Result<Self> {
        #[cfg(not(test))]
        let executable =
            std::env::current_exe().map_err(|_| error("Cannot locate process watchdog."))?;
        #[cfg(test)]
        let executable = crate::agent_tests::fixture_executable();
        let mut child = Command::new(executable)
            .arg("--forma-watch-agent")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| error("Cannot launch process watchdog."))?;
        let mut input = child
            .stdin
            .take()
            .ok_or_else(|| error("Cannot initialize process watchdog."))?;
        writeln!(input, "{group}").map_err(|_| error("Cannot arm process watchdog."))?;
        let stdout = child.stdout.take();
        let mut guard = Self {
            child,
            input: Some(input),
        };
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let ready = stdout.is_some_and(|mut out| {
                let mut b = [0];
                out.read_exact(&mut b).is_ok() && b == *b"R"
            });
            let _ = tx.send(ready);
        });
        if rx.recv_timeout(Duration::from_secs(5)) != Ok(true) {
            guard.input.take();
            let _ = guard.child.kill();
            return Err(error(
                "Process watchdog did not initialize; execution stopped.",
            ));
        }
        Ok(guard)
    }
    fn finish(&mut self) {
        if let Some(mut input) = self.input.take() {
            let _ = writeln!(input, "done");
        }
        let _ = self.child.wait();
    }
}
#[cfg(unix)]
impl Drop for Watchdog {
    fn drop(&mut self) {
        self.input.take();
        let _ = self.child.wait();
    }
}
/// Internal pipe watchdog: EOF from the owning Forma process kills the approved
/// agent group. No shell, paths or user-supplied commands are accepted here.
#[cfg(feature = "desktop")]
pub fn watchdog_entry() -> bool {
    if std::env::args().nth(1).as_deref() != Some("--forma-watch-agent") {
        return false;
    }
    #[cfg(unix)]
    {
        use std::io::BufRead;
        let mut lines = std::io::stdin().lock().lines();
        if let Some(Ok(group)) = lines.next() {
            if let Ok(group) = group.parse::<i32>() {
                let _ = std::io::stdout().write_all(b"R");
                let _ = std::io::stdout().flush();
                if group > 1
                    && lines
                        .next()
                        .is_none_or(|line| !matches!(line.as_deref(), Ok("done")))
                {
                    // SAFETY: positive pgid created by the owner; kill cannot
                    // affect processes belonging to another OS user.
                    unsafe {
                        libc::kill(-group, libc::SIGKILL);
                    }
                }
            }
        }
    }
    true
}
