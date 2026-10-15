#!/usr/bin/env python3
"""Launch a packaged app with isolated data on a native runner.

Windows requires a fresh GitHub-hosted runner and installs NSIS silently into a temporary folder. Linux extracts a deb;
macOS mounts a DMG and copies its app. A running process plus a valid database
proves initialization only, never interactive installer or rendering correctness.
"""
import argparse
import os
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
args = parser.parse_args()
bundle = args.bundle.resolve()
package_format = args.format or ('nsis' if sys.platform == 'win32' else 'dmg' if sys.platform == 'darwin' else 'deb')
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
                                with sqlite3.connect(databases[0]) as conn:
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
                finally:
                    stop(process)
            if launch == 0:
                # Isolated database fixture, not a claim of UI-driven persistence.
                with sqlite3.connect(database) as conn:
                    conn.execute("INSERT INTO workspaces(id,name,created_at,updated_at) VALUES(?,?,?,?)",
                                 (marker, 'Package restart fixture', '2026-10-15T00:00:00Z', '2026-10-15T00:00:00Z'))
            else:
                with sqlite3.connect(database) as conn:
                    assert conn.execute('SELECT name FROM workspaces WHERE id=?', (marker,)).fetchone() == ('Package restart fixture',)
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
        print(f'PASS: {method}; two launches, schema 6/integrity/foreign keys and seeded restart fixture preserved.'
              ' Windows uninstall/data retention checked when applicable.'
              ' Interactive UI, native installer dialogs, signing trust and notarization remain unverified.')
    finally:
        if sys.platform == 'win32' and data.exists():
            # Only reached after the fresh hosted directory precondition passed.
            shutil.rmtree(data)
