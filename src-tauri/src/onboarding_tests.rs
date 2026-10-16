use crate::{
    agent_provider as engine,
    agent_tests::{fixture_executable, setup, wait},
    onboarding::*,
};
use std::path::{Path, PathBuf};
fn status(exit: Option<i32>, log: &str) -> engine::Execution {
    engine::Execution {
        status: if exit == Some(0) {
            "Completed"
        } else {
            "Failed"
        },
        exit,
        result: "",
        log: log.into(),
    }
}
#[test]
fn authentication_is_explicit_and_unknown_responses_fail_closed() {
    assert_eq!(
        auth_result("codex", &status(Some(0), "Logged in using ChatGPT")),
        "authenticated"
    );
    assert_eq!(
        auth_result("codex", &status(Some(1), "Not logged in")),
        "unauthenticated"
    );
    assert_eq!(
        auth_result("codex", &status(Some(0), "Something new")),
        "unknown"
    );
    assert_eq!(
        auth_result("codex", &status(Some(2), "Logged in")),
        "unknown"
    );
    assert_eq!(
        auth_result("claude", &status(Some(0), r#"{"loggedIn":true}"#)),
        "authenticated"
    );
    assert_eq!(
        auth_result("claude", &status(Some(1), r#"{"loggedIn":false}"#)),
        "unauthenticated"
    );
    for (exit, log) in [
        (Some(0), r#"{"loggedIn":false}"#),
        (Some(0), "not JSON"),
        (None, ""),
        (Some(1), r#"{"loggedIn":true}"#),
    ] {
        assert_eq!(auth_result("claude", &status(exit, log)), "unknown");
    }
}
#[test]
fn all_platform_search_policies_ignore_relative_path_and_wrappers() {
    for platform in ["linux", "macos", "windows"] {
        let home = if cfg!(windows) {
            Path::new("C:\\Users\\fixture")
        } else {
            Path::new("/fixture")
        };
        let dirs = vec![
            PathBuf::from("relative"),
            home.join("bin"),
            home.join("bin"),
        ];
        let candidates = locations(platform, "codex", Some(home), Some(home), dirs);
        let name = if platform == "windows" {
            "codex.exe"
        } else {
            "codex"
        };
        assert!(candidates.contains(&home.join("bin").join(name)));
        assert!(candidates.contains(&home.join(".local/bin").join(name)));
        assert_eq!(
            candidates
                .iter()
                .filter(|c| **c == home.join("bin").join(name))
                .count(),
            1
        );
        assert!(candidates
            .iter()
            .all(|p| p.is_absolute() && !p.to_string_lossy().ends_with(".cmd")));
        if platform == "windows" {
            assert!(candidates.contains(&home.join("Programs/OpenAI/Codex/bin/codex.exe")));
        }
        if platform == "macos" && !cfg!(windows) {
            assert!(candidates.contains(&PathBuf::from("/opt/homebrew/bin/codex")));
        }
    }
}
fn candidate(s: &mut crate::service::Service, path: &Path) -> String {
    let path = engine::executable(path).unwrap();
    let c = Candidate {
        token: uuid::Uuid::new_v4().to_string(),
        path: path.to_string_lossy().into_owned(),
        sha256: engine::digest(&path).unwrap(),
    };
    s.agents.discovered.insert(
        c.token.clone(),
        Discovered {
            provider: "codex".into(),
            candidate: c.clone(),
            created: chrono::Utc::now().timestamp(),
        },
    );
    c.token
}
#[test]
fn discovered_identity_expiry_and_explicit_authorization_are_required() {
    let (dir, mut s, _, _) = setup("Onboarding");
    let copy = dir.path().join(if cfg!(windows) {
        "selected.exe"
    } else {
        "selected"
    });
    std::fs::copy(fixture_executable(), &copy).unwrap();
    let token = candidate(&mut s, &copy);
    assert!(s.agent_authorize_discovered(&token, false).is_err());
    std::fs::write(&copy, "changed bytes").unwrap();
    assert!(s.agent_authorize_discovered(&token, true).is_err());
    std::fs::copy(fixture_executable(), &copy).unwrap();
    let token = candidate(&mut s, &copy);
    s.agents.discovered.get_mut(&token).unwrap().created -= 301;
    assert!(s.agent_authorize_discovered(&token, true).is_err());
    assert!(s
        .agent_authorize_discovered("/arbitrary/path", true)
        .is_err());
    let token = candidate(&mut s, &copy);
    s.agent_authorize_discovered(&token, true).unwrap();
    assert!(s.agent_authorize_discovered(&token, true).is_err());
    assert!(
        s.agent_settings()
            .unwrap()
            .iter()
            .find(|p| p.provider == "codex")
            .unwrap()
            .trusted
    );
}
#[test]
fn onboarding_rescan_authentication_and_first_task_use_existing_coordinator() {
    let (dir, mut s, project, issue) = setup("Onboarding first task");
    let status = s.agent_onboarding("codex").unwrap();
    assert!(status.authorized && status.compatible && status.ready);
    assert_eq!(status.authentication, "authenticated");
    assert!(s.agent_runs(&project, &issue.id, 0).unwrap().is_empty());
    assert!(!dir.path().join("agent-worktrees").exists());
    assert!(s.agent_login("codex", false).is_err());
    assert!(s.agent_onboarding("arbitrary").is_err());
    let preview = s.agent_preview(&project, &issue.id, "codex").unwrap();
    assert!(s.agent_start(&preview.token, false, false).is_err());
    let run = s.agent_start(&preview.token, true, false).unwrap();
    assert_eq!(wait(&s, &run.id).status, "Completed");
    assert!(!dir.path().join("repo/agent-result.txt").exists());
    assert!(Path::new(&run.worktree).join("agent-result.txt").exists());
    let mut settings = s
        .agent_settings()
        .unwrap()
        .into_iter()
        .find(|p| p.provider == "codex")
        .unwrap();
    settings.require_os_isolation = true;
    s.agent_save_settings(settings).unwrap();
    assert!(!s.agent_onboarding("codex").unwrap().ready);
    drop(s);
    let mut s = crate::service::Service::open(&dir.path().join("test.sqlite3")).unwrap();
    assert!(s.agents.discovered.is_empty());
    assert!(!s.agent_onboarding("codex").unwrap().ready);
}
#[test]
fn login_is_explicit_bounded_cancellable_and_not_persisted() {
    let (_dir, mut s, _, _) = setup("Login");
    s.agent_login("codex", true).unwrap();
    s.agent_cancel_login().unwrap();
    let login = s.agents.login.as_ref().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if *login.state.lock().unwrap() != "running" {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    assert_eq!(*login.state.lock().unwrap(), "cancelled");
}

#[test]
fn discovery_does_not_execute_candidates_and_rescans_missing_or_incompatible_files() {
    let dir = tempfile::tempdir().unwrap();
    let exe = dir
        .path()
        .join(if cfg!(windows) { "codex.exe" } else { "codex" });
    assert!(scan_files(vec![exe.clone()]).is_empty());
    // An executable shell script would create a marker if discovery ran it.
    std::fs::write(&exe, "#!/bin/sh\ntouch discovery-executed\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    let scanned = scan_files(vec![exe.clone(), exe.clone()]);
    assert_eq!(scanned.len(), 1);
    assert!(!dir.path().join("discovery-executed").exists());
    assert!(!Path::new("discovery-executed").exists());
    std::fs::remove_file(&exe).unwrap();
    assert!(scan_files(vec![exe.clone()]).is_empty());
    // A real native executable that identifies as Claude is incompatible with Codex.
    let wrong = dir.path().join(if cfg!(windows) {
        "claude-wrong.exe"
    } else {
        "claude-wrong"
    });
    std::fs::copy(fixture_executable(), &wrong).unwrap();
    let mut service = crate::service::Service::open(&dir.path().join("db")).unwrap();
    assert!(service.agent_authorize("codex", &wrong).is_err());
    let result = service.agent_onboarding("codex").unwrap();
    assert!(!result.authorized && !result.compatible && !result.ready);
    service.agent_authorize("claude", &wrong).unwrap();
    let result = service.agent_onboarding("claude").unwrap();
    assert!(result.authorized && result.compatible);
    assert_eq!(result.authentication, "unauthenticated");
    assert!(!result.ready);
    let old_cli = dir.path().join(if cfg!(windows) {
        "codex-incompatible.exe"
    } else {
        "codex-incompatible"
    });
    std::fs::copy(fixture_executable(), &old_cli).unwrap();
    let error = service.agent_authorize("codex", &old_cli).unwrap_err();
    assert!(error.message.contains("lacks required security options"));
    assert!(!service.agent_onboarding("codex").unwrap().ready);
}
