#!/usr/bin/env python3
"""Opt-in Windows/macOS native first-task smoke with a compiled simulated CLI.

Uses public UI Automation/AX only and read-only SQLite assertions. Windows is
restricted to fresh GitHub-hosted profiles because known-folder APIs ignore env
app-data overrides. macOS never requests or bypasses TCC permissions. Exit 77
means unavailable accessibility permission, never a successful onboarding test.
"""
import json
import os
import platform
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import time
import tempfile
from onboarding_metrics import OnboardingMetrics


def run(binary):
    if sys.platform not in ('win32', 'darwin'):
        raise RuntimeError('Use native-onboarding-smoke.py on Linux')
    root = Path(tempfile.mkdtemp(prefix='forma-m09-native-'))
    print('Artifacts:', root, flush=True)
    env = os.environ.copy()
    home = root / 'home'
    home.mkdir()
    if sys.platform == 'win32':
        if env.get('GITHUB_ACTIONS') != 'true' or env.get('RUNNER_ENVIRONMENT') != 'github-hosted':
            raise RuntimeError('Requires a disposable GitHub-hosted Windows profile')
        roaming = subprocess.check_output(['powershell', '-NoProfile', '-Command',
            "[Environment]::GetFolderPath('ApplicationData')"], text=True).strip()
        data = Path(roaming) / 'dev.forma.desktop'
    else:
        env['HOME'] = str(home)
        data = home / 'Library/Application Support/dev.forma.desktop'
    if data.exists() or data.is_symlink():
        raise RuntimeError('Refusing existing Forma app data')
    providers = root / 'provider'
    providers.mkdir()
    fixture = providers / ('codex.exe' if sys.platform == 'win32' else 'codex')
    subprocess.run(['rustc', '--edition=2021', str(Path(__file__).with_name('agent-fixture.rs')), '-o', str(fixture)], check=True)
    env['CODEX_INSTALL_DIR'] = str(providers)
    repo = root / 'repository'
    subprocess.run(['git', 'init', '-b', 'main', str(repo)], check=True)
    (repo / 'tracked').write_text('original\n')
    subprocess.run(['git', '-C', str(repo), 'add', 'tracked'], check=True)
    subprocess.run(['git', '-C', str(repo), '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-m', 'Initial'], check=True)
    metrics = OnboardingMetrics()
    db = data / 'forma.sqlite3'
    def rows(query):
        with sqlite3.connect(f'file:{db.as_posix()}?mode=ro', uri=True) as connection:
            return connection.execute(query).fetchall()
    with (root / 'native.log').open('w') as log:
        process = subprocess.Popen([str(binary)], env=env, stdout=log, stderr=log)
        try:
            driver = Path(__file__).with_name('onboarding-ui-windows.ps1' if sys.platform == 'win32' else 'onboarding-ui-macos.swift')
            identity = str(fixture.resolve())
            windows_prefix = chr(92) * 2 + '?' + chr(92)
            if sys.platform == 'win32' and not identity.startswith(windows_prefix):
                identity = windows_prefix + identity
            command = (['powershell', '-NoProfile', '-File'] if sys.platform == 'win32' else ['swift']) + [str(driver), str(process.pid), identity, str(repo), str(root / 'metrics.json')]
            result = subprocess.run(command, env=env, timeout=240)
            if result.returncode == 77:
                metrics.result = 'unverified-permission'
                return 77
            result.check_returncode()
            deadline = time.monotonic() + 10
            while rows("SELECT review FROM agent_runs") != [('accepted',)] and time.monotonic() < deadline:
                time.sleep(.2)
            assert rows("SELECT status, exit_code, review FROM agent_runs") == [('Completed', 0, 'accepted')]
            worktree = Path(rows('SELECT worktree FROM agent_runs')[0][0])
            assert (worktree / 'agent-result.txt').read_text() == 'simulated provider change\n'
            assert not (repo / 'agent-result.txt').exists()
            assert rows('PRAGMA integrity_check') == [('ok',)]
            assert rows('PRAGMA foreign_key_check') == []
            metrics.result = 'passed'
            print('PASS: native onboarding, approved simulated task, review and database integrity', flush=True)
        except Exception:
            metrics.errors += 1
            raise
        finally:
            # The Python result includes database/worktree verification after UI actions.
            metrics_file = root / 'metrics.json'
            if metrics_file.exists():
                measured = json.loads(metrics_file.read_text(encoding='utf-8-sig'))
                measured['result'] = metrics.result
                measured['errors'] += metrics.errors
                metrics_file.write_text(json.dumps(measured, indent=2) + '\n')
            else:
                metrics.write(metrics_file)
            output = Path('package-evidence')
            output.mkdir(exist_ok=True)
            shutil.copyfile(metrics_file, output / f'onboarding-{sys.platform}-{platform.machine()}.json')
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=20)
            # Remove only data created by this invocation on ephemeral Windows.
            if sys.platform == 'win32' and data.exists():
                shutil.rmtree(data)
    return 0


if __name__ == '__main__':
    sys.exit(run(Path(sys.argv[1]).resolve()))
