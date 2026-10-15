#!/usr/bin/env python3
"""Launch a packaged app with isolated data on a native runner.

Windows requires a fresh GitHub-hosted runner and installs NSIS silently into a temporary folder. Linux extracts a deb;
macOS mounts a DMG and copies its app. A running process plus a valid database
proves initialization only, never interactive installer or rendering correctness.
"""
import argparse
from datetime import datetime, timezone
import platform
from contextlib import closing
import os
import plistlib
import uuid
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('bundle', type=Path)
parser.add_argument('--format', choices=['deb', 'appimage', 'nsis', 'msi', 'dmg'])
parser.add_argument('--ui', action='store_true', help='Use public accessibility APIs; report macOS TCC unavailability explicitly')
parser.add_argument('--install-deb', action='store_true', help='Install/remove DEB only on a fresh ephemeral GitHub-hosted Linux runner')
args = parser.parse_args()
bundle = args.bundle.resolve()
started = datetime.now(timezone.utc).isoformat()
print(f'Host: {platform.platform()} / {platform.machine()}; started {started}', flush=True)
package_format = args.format or ('nsis' if sys.platform == 'win32' else 'dmg' if sys.platform == 'darwin' else 'deb')
if args.install_deb and (not sys.platform.startswith('linux') or package_format != 'deb'):
    parser.error('--install-deb requires Linux and DEB format')
with tempfile.TemporaryDirectory(prefix="forma-package-smoke-") as temporary:
    root = Path(temporary)
    env = os.environ.copy()
    data = root / "data"
    data.mkdir()
    if sys.platform != "win32":
        env.update(HOME=str(data), XDG_DATA_HOME=str(data),
                   XDG_CONFIG_HOME=str(data / "config"))
    if sys.platform == "win32":
        # Tauri uses SHGetKnownFolderPath, which ignores APPDATA overrides.
        # Use only a fresh, ephemeral hosted profile; never launch against an
        # existing Forma directory or a developer's normal Windows profile.
        if os.environ.get("GITHUB_ACTIONS") != "true" or os.environ.get("RUNNER_ENVIRONMENT") != "github-hosted":
            raise RuntimeError("Windows package smoke requires an ephemeral GitHub-hosted runner")
        roaming = subprocess.check_output([
            "powershell", "-NoProfile", "-NonInteractive", "-Command",
            "[Environment]::GetFolderPath('ApplicationData')",
        ], text=True, timeout=30).strip()
        if not roaming or not Path(roaming).is_absolute():
            raise RuntimeError("Cannot resolve the Windows roaming app-data known folder")
        data = Path(roaming) / "dev.forma.desktop"
        if data.exists() or data.is_symlink():
            raise RuntimeError("Refusing to launch with pre-existing Forma app data")
        install = root / "installed"
        if package_format == 'msi':
            installer = next(bundle.glob('msi/*.msi'))
            subprocess.run(['msiexec', '/i', str(installer), '/qn', '/norestart', f'INSTALLDIR={install}', '/L*v', str(root / 'msi.log')], check=True, timeout=180)
        else:
            installer = next(bundle.glob('nsis/*.exe'))
            subprocess.run([str(installer), '/S', f'/D={install}'], check=True, timeout=180)
        executable = next(install.rglob('forma.exe'))
        method = f'{package_format.upper()} silent install in fresh hosted app data'
    elif sys.platform == "darwin":
        image = next(bundle.glob("dmg/*.dmg"))
        mount = root / "mounted"
        mount.mkdir()
        subprocess.run(["hdiutil", "attach", "-readonly", "-nobrowse", "-mountpoint", str(mount), str(image)], check=True)
        try:
            source = next(mount.glob("*.app"))
            copied = root / source.name
            shutil.copytree(source, copied, symlinks=True)
        finally:
            subprocess.run(["hdiutil", "detach", str(mount)], check=True)
        executable = copied / "Contents/MacOS/forma"
        method = "DMG mount/copy and bundled process initialization"
    elif package_format == 'appimage':
        image = next(bundle.glob('appimage/*.AppImage'))
        image.chmod(image.stat().st_mode | 0o100)
        subprocess.run([str(image), '--appimage-extract'], cwd=root, check=True, stdout=subprocess.DEVNULL, timeout=120)
        executable = root / 'squashfs-root/AppRun'
        method = 'AppImage extraction/runtime execution (FUSE-free; not cross-distro portability)'
    elif args.install_deb:
        if package_format != 'deb' or os.environ.get('GITHUB_ACTIONS') != 'true' or os.environ.get('RUNNER_ENVIRONMENT') != 'github-hosted':
            raise RuntimeError('DEB installation requires an ephemeral GitHub-hosted Linux runner')
        executable = Path('/usr/bin/forma')
        if executable.exists() or executable.is_symlink():
            raise RuntimeError('Refusing to replace a pre-existing Forma installation')
        package = next(bundle.glob('deb/*.deb'))
        subprocess.run(['sudo', 'apt-get', 'install', '-y', str(package)], check=True, timeout=180)
        method = 'DEB apt installation and installed process initialization'
    else:
        package = next(bundle.glob("deb/*.deb"))
        install = root / "extracted"
        if shutil.which("dpkg-deb"):
            subprocess.run(["dpkg-deb", "-x", str(package), str(install)], check=True)
        else:
            # Arch development hosts may provide bsdtar rather than dpkg-deb.
            archive = root / "deb-archive"
            archive.mkdir()
            install.mkdir()
            subprocess.run(["bsdtar", "-xf", str(package), "-C", str(archive)], check=True)
            payload = next(archive.glob("data.tar.*"))
            subprocess.run(["bsdtar", "-xf", str(payload), "-C", str(install)], check=True)
        executable = install / "usr/bin/forma"
        method = "deb extraction and packaged process initialization (no installation)"
    if args.ui and sys.platform.startswith('linux'):
        env.update(GTK_MODULES='gail:atk-bridge', NO_AT_BRIDGE='0')
        # Fresh Xvfb session only; do not alter an existing desktop WM.
        if subprocess.run(['wmctrl', '-m'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL).returncode:
            wm = subprocess.Popen(['openbox'], env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            time.sleep(1)
    if sys.platform == 'darwin':
        info = plistlib.loads((copied / 'Contents/Info.plist').read_bytes())
        assert info['LSMinimumSystemVersion'] == '15.0', info
        assert info['CFBundleIdentifier'] == 'dev.forma.desktop', info
        assert info['CFBundleShortVersionString'] == '0.1.0-beta.1', info
        print('PASS: DMG application metadata, macOS deployment floor 15.0')
    if sys.platform == 'win32':
        runtime = subprocess.check_output(['powershell', '-NoProfile', '-Command',
            r"$p=@('HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'); Get-ItemProperty $p -ErrorAction SilentlyContinue | Select-Object -ExpandProperty pv"], text=True).strip()
        # Runtime initialization below is authoritative; registry discovery is diagnostic.
        print('WebView2 registered version(s): ' + (runtime or 'not discovered; see launch result'))

    def ui(process, phase):
        if sys.platform == 'win32':
            command = ['powershell', '-NoProfile', '-File', str(Path(__file__).with_name('package-ui-windows.ps1')), str(process.pid), phase, marker]
        elif sys.platform == 'darwin':
            command = ['swift', str(Path(__file__).with_name('package-ui-macos.swift')), str(process.pid)]
        else:
            command = ['/usr/bin/python3', str(Path(__file__).with_name('package-ui-linux.py')), str(process.pid), phase, marker]
        result = subprocess.run(command, env=env, timeout=120)
        if sys.platform == 'darwin' and result.returncode in (0, 77):
            # NSRunningApplication termination does not require AX/TCC permission.
            subprocess.run(['swift', '-e', 'import AppKit; guard let app = NSRunningApplication(processIdentifier: Int32(CommandLine.arguments[1])!), app.terminate() else { exit(1) }', str(process.pid)], check=True, timeout=60)
        else:
            result.check_returncode()
        process.wait(timeout=20)
        assert process.returncode == 0, f'Nonzero graceful exit: {process.returncode}'
        print('PASS: ordinary native close exited 0')

    def stop(process):
        if process.poll() is None:
            if sys.platform == 'win32':
                subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], check=True, timeout=30)
            else:
                process.terminate()
            try:
                process.wait(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=5)

    marker = str(uuid.uuid4())
    database = None
    try:
        for launch in range(2):
            log = root / f'process-{launch}.log'
            with log.open('w') as output:
                process = subprocess.Popen([str(executable)], env=env, stdout=output, stderr=subprocess.STDOUT)
                try:
                    deadline = time.monotonic() + 30
                    initialized = False
                    while time.monotonic() < deadline:
                        if process.poll() is not None:
                            raise RuntimeError(f'Packaged app exited {process.returncode}: {log.read_text()}')
                        databases = list(data.rglob('forma.sqlite3'))
                        if databases:
                            try:
                                with closing(sqlite3.connect(databases[0])) as conn:
                                    if conn.execute('PRAGMA user_version').fetchone() == (6,):
                                        assert conn.execute('PRAGMA integrity_check').fetchone() == ('ok',)
                                        assert conn.execute('PRAGMA foreign_key_check').fetchall() == []
                                        database = databases[0]
                                        initialized = True
                                        break
                            except sqlite3.Error:
                                pass
                        time.sleep(.1)
                    if not initialized:
                        raise RuntimeError(f'No initialized isolated schema-6 database: {log.read_text()}')
                    time.sleep(2)
                    assert process.poll() is None, log.read_text()
                    if args.ui:
                        ui(process, 'create' if launch == 0 else 'restart')
                finally:
                    stop(process)
            if args.ui and sys.platform != 'darwin':
                with closing(sqlite3.connect(database)) as conn:
                    assert conn.execute('SELECT name FROM workspaces WHERE name=?', (marker,)).fetchone() == (marker,)
            if launch == 0:
                # Isolated database fixture, not a claim of UI-driven persistence.
                with closing(sqlite3.connect(database)) as conn:
                    conn.execute("INSERT INTO workspaces(id,name,created_at,updated_at) VALUES(?,?,?,?)",
                                 (marker, 'Package restart fixture', '2026-10-15T00:00:00Z', '2026-10-15T00:00:00Z'))
                    conn.commit()
            else:
                with closing(sqlite3.connect(database)) as conn:
                    assert conn.execute('SELECT name FROM workspaces WHERE id=?', (marker,)).fetchone() == ('Package restart fixture',)
        if args.install_deb:
            subprocess.run(['sudo', 'apt-get', 'remove', '-y', 'forma'], check=True, timeout=120)
            assert not executable.exists(), 'DEB uninstall retained executable'
            assert database.exists(), 'DEB uninstall removed persistent data'
        if sys.platform == 'win32':
            if package_format == 'msi':
                subprocess.run(['msiexec', '/x', str(installer), '/qn', '/norestart'], check=True, timeout=180)
            else:
                uninstaller = next(install.glob('*ninstall*.exe'))
                subprocess.run([str(uninstaller), '/S'], check=True, timeout=120)
            deadline = time.monotonic() + 30
            while executable.exists() and time.monotonic() < deadline:
                time.sleep(.2)
            assert not executable.exists(), 'Uninstall retained executable'
            assert database.exists(), 'Uninstall removed persistent data'
    finally:
        if os.environ.get('GITHUB_ACTIONS') == 'true':
            evidence = Path(os.environ['GITHUB_WORKSPACE']) / 'package-evidence' / package_format
            evidence.mkdir(parents=True, exist_ok=True)
            for path in root.glob('*.log'):
                # Only sample package-run logs; never copy databases, backups or cores.
                shutil.copy2(path, evidence / path.name)
            (evidence / 'host.txt').write_text(f'{platform.platform()} / {platform.machine()}\nStarted {started}\n')
            if sys.platform.startswith('linux'):
                with (evidence / 'journal.log').open('w') as output:
                    subprocess.run(['journalctl', '--since', started, '--no-pager'], stdout=output, stderr=subprocess.STDOUT, timeout=30)
                journal = (evidence / 'journal.log').read_text()
                if 'dumped core' in journal and ('WebKit' in journal or 'forma' in journal):
                    raise RuntimeError('Native crash metadata observed; inspect package-evidence journal')
        if sys.platform == 'win32' and data.exists():
            # Only reached after the fresh hosted directory precondition passed.
            shutil.rmtree(data)

    print(f'PASS: {method}; two launches, schema 6/integrity/foreign keys and seeded restart fixture preserved.'
          ' Installed DEB/Windows uninstall and data retention checked when applicable.'
          ' Accessibility scope is reported separately; native installer dialogs, signing trust and notarization remain unverified.')
