#!/usr/bin/env python3
"""Launch a packaged app with isolated data on a native runner.

Windows installs NSIS silently into a temporary folder. Linux extracts a deb;
macOS mounts a DMG and copies its app. A running process plus a valid database
proves initialization only, never interactive installer or rendering correctness.
"""
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import time

bundle = Path(sys.argv[1]).resolve()
with tempfile.TemporaryDirectory(prefix="forma-package-smoke-") as temporary:
    root = Path(temporary)
    env = os.environ.copy()
    data = root / "data"
    data.mkdir()
    env.update(HOME=str(data), USERPROFILE=str(data), APPDATA=str(data),
               LOCALAPPDATA=str(data), XDG_DATA_HOME=str(data),
               XDG_CONFIG_HOME=str(data / "config"))
    if sys.platform == "win32":
        installer = next(bundle.glob("nsis/*.exe"))
        install = root / "installed"
        subprocess.run([str(installer), "/S", f"/D={install}"], check=True, timeout=120)
        executable = next(install.rglob("forma.exe"))
        method = "NSIS silent install and installed process initialization"
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
    log = root / "process.log"
    with log.open("w") as output:
        process = subprocess.Popen([str(executable)], env=env, stdout=output, stderr=subprocess.STDOUT)
        try:
            deadline = time.monotonic() + 30
            database = None
            while time.monotonic() < deadline:
                if process.poll() is not None:
                    raise RuntimeError(f"Packaged app exited {process.returncode}: {log.read_text()}")
                databases = list(data.rglob("forma.sqlite3"))
                if databases:
                    try:
                        with sqlite3.connect(databases[0]) as conn:
                            if conn.execute("PRAGMA user_version").fetchone() == (6,):
                                assert conn.execute("PRAGMA integrity_check").fetchone() == ("ok",)
                                database = databases[0]
                                break
                    except sqlite3.Error:
                        pass
                time.sleep(.1)
            if database is None:
                raise RuntimeError(f"No initialized isolated schema-6 database: {log.read_text()}")
            time.sleep(2)
            assert process.poll() is None, log.read_text()
            print(f"PASS: {method}; schema 6 integrity ok. Interactive installation, rendering, signing trust and notarization remain unverified.")
        finally:
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)
