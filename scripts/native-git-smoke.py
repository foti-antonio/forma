#!/usr/bin/env python3
"""Opt-in Linux/Hyprland smoke of real forms, IPC and SQLite persistence.

Requires a built desktop executable, wtype, grim and tesseract. Run in a graphical
session and leave the test app focused. Uses isolated temporary app-data; never
writes SQL or uses an application test backdoor. Screenshots/logs remain in the
printed temporary directory. Pass a binary path as the first argument.
"""
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time

binary = Path(sys.argv[1] if len(sys.argv) > 1 else "src-tauri/target/debug/forma").resolve()
root = Path(tempfile.mkdtemp(prefix="forma-m04-git-smoke-"))
database = root / "data/dev.forma.desktop/forma.sqlite3"
env = dict(os.environ, XDG_DATA_HOME=str(root / "data"),
           XDG_CONFIG_HOME=str(root / "config"), XDG_CACHE_HOME=str(root / "cache"))
process = None
native_dialog = None
log = (root / "native.log").open("w")


def until(action, description, timeout=20):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        if process.poll() is not None:
            raise RuntimeError(f"Native app exited ({process.returncode}); see {root}")
        if action():
            return
        time.sleep(0.2)
    raise RuntimeError(f"Timed out: {description}; see {root}")


def active():
    return json.loads(subprocess.check_output(["hyprctl", "-j", "activewindow"]))


def focused():
    window = active()
    return window.get("pid") == process.pid or (
        native_dialog is not None and window.get("title") == native_dialog
        and window.get("class") in ("xdg-desktop-portal-gtk", "org.freedesktop.impl.portal.desktop.gtk")
    )


def keys(*args):
    if not focused():
        raise RuntimeError("Test app lost focus; refusing to send keys to another app")
    subprocess.run(["wtype", "-s", "100", *args, "-s", "100"], check=True)
    time.sleep(0.25)


def tabs(count, backwards=False):
    for _ in range(count):
        keys(*(["-M", "shift", "-k", "Tab", "-m", "shift"] if backwards else ["-k", "Tab"]))


def text(value):
    keys("-d", "30", value)


def screenshot(name="current"):
    if not focused():
        raise RuntimeError("Test app lost focus")
    w = active()
    x, y = w["at"]
    width, height = w["size"]
    path = root / (name + ".png")
    subprocess.run(["grim", "-g", f"{x},{y} {width}x{height}", str(path)], check=True)
    return path


def visible(value):
    def check():
        result = subprocess.check_output(["tesseract", str(screenshot()), "stdout"], stderr=subprocess.DEVNULL, text=True)
        return value.lower() in result.lower()
    until(check, "visible text: " + value)


def rows(sql):
    if not database.exists():
        return []
    with sqlite3.connect(f"file:{database}?mode=ro", uri=True) as connection:
        return connection.execute(sql).fetchall()


def assert_snapshot_matches(path):
    with sqlite3.connect(f"file:{database}?mode=ro", uri=True) as live, \
            sqlite3.connect(f"file:{path}?mode=ro", uri=True) as backup:
        tables = [r[0] for r in live.execute("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")]
        for table in tables:
            identifier = '"' + table.replace('"', '""') + '"'
            assert sorted(live.execute(f"SELECT * FROM {identifier}").fetchall(), key=repr) == sorted(
                backup.execute(f"SELECT * FROM {identifier}").fetchall(), key=repr), table


def launch():
    global process
    process = subprocess.Popen([str(binary)], env=env, stdout=log, stderr=log)
    until(focused, "native window focus")
    visible("Projects")
    time.sleep(0.5)


def stop():
    process.terminate()
    process.wait(timeout=15)


def save():
    keys("-M", "ctrl", "-k", "Return", "-m", "ctrl")


def search(value):
    # Start from a button, then use the project's search shortcut.
    keys("/")
    text(value)
    time.sleep(0.6)


print(f"Native Git smoke artifacts: {root}", flush=True)
repo = root / "repository"
repo.mkdir()
subprocess.run(["git", "init", "-b", "main", str(repo)], check=True, stdout=subprocess.DEVNULL)
(repo / "tracked").write_text("original\n")
for args in [["add", "tracked"], ["-c", "user.name=Smoke Author", "-c", "user.email=smoke@example.test", "commit", "-m", "Fix F1-1"], ["branch", "feature/F1-1-work"]]:
    subprocess.run(["git", "-C", str(repo), *args], check=True, stdout=subprocess.DEVNULL)
(repo / "untracked").write_text("new\n")
index_before = (repo / ".git/index").read_bytes()
head_before = (repo / ".git/HEAD").read_bytes()
try:
    launch()
    tabs(3)
    keys("-k", "Return")
    visible("Workspace name")
    text("Git smoke studio")
    keys("-k", "Return")
    until(lambda: bool(rows("SELECT id FROM workspaces")), "workspace")
    time.sleep(.5)
    tabs(1, backwards=True)
    keys("-k", "Return")
    visible("Project name")
    text("Git smoke engine")
    keys("-k", "Return")
    until(lambda: bool(rows("SELECT id FROM projects")), "project")
    time.sleep(.5)
    tabs(5)
    keys("-k", "Return")
    visible("active issues")
    keys("n")
    visible("What needs to happen")
    text("Git linked issue")
    save()
    until(lambda: bool(rows("SELECT id FROM issues")), "issue")
    time.sleep(.5)
    keys("/")
    tabs(4, backwards=True)
    keys("-k", "Return")
    tabs(1)
    native_dialog = "Connect Git repository"
    keys("-k", "Return", "-s", "1800", "-M", "ctrl", "-k", "l", "-m", "ctrl",
         "-M", "ctrl", "-k", "a", "-m", "ctrl", "-s", "300", "-d", "30", str(repo), "-s", "500", "-M", "alt", "-k", "s", "-m", "alt")
    screenshot("folder-result")
    until(lambda: bool(rows("SELECT id FROM git_repositories WHERE connected=1")), "native Git connection")
    until(lambda: active().get("pid") == process.pid, "return from folder picker")
    native_dialog = None
    visible("Repository readable")
    screenshot("connected")
    keys("/")
    text("Git linked issue")
    time.sleep(.5)
    tabs(14)
    keys("-k", "Return")
    visible("Edit issue")
    tabs(16)
    keys("-k", "Return")
    time.sleep(.8)
    tabs(2)
    keys("-k", "Return")
    assert rows("SELECT count(*) FROM git_links") == [(0,)]
    tabs(6)
    screenshot("branch-confirmation")
    keys("-k", "Return")
    until(lambda: rows("SELECT count(*) FROM git_links") == [(1,)], "confirmed branch association")
    screenshot("linked-branch")
    keys("-k", "Escape")
    time.sleep(.4)
    keys("/")
    tabs(14)
    keys("-k", "Return")
    visible("Edit issue")
    tabs(16)
    keys("-k", "Return")
    time.sleep(.8)
    tabs(4)
    keys("-k", "Return")
    tabs(5)
    keys("-k", "Return")
    until(lambda: rows("SELECT count(*) FROM git_links") == [(2,)], "confirmed commit association")
    keys("-k", "Escape")
    stop()
    launch()
    tabs(5)
    keys("-k", "Return")
    visible("active issues")
    keys("/")
    text("Git linked issue")
    time.sleep(.5)
    tabs(14)
    keys("-k", "Return")
    visible("Edit issue")
    tabs(16)
    keys("-k", "Return")
    time.sleep(.8)
    tabs(2)
    screenshot("restarted-links")
    assert rows("SELECT kind FROM git_links ORDER BY kind") == [("branch",), ("commit",)]
    assert rows("PRAGMA integrity_check") == [("ok",)]
    assert rows("PRAGMA foreign_key_check") == []
    assert (repo / ".git/index").read_bytes() == index_before
    assert (repo / ".git/HEAD").read_bytes() == head_before
    assert (repo / "tracked").read_text() == "original\n"
    print("PASS: native folder connection, overview, explicit branch/commit confirmation, restart persistence and unchanged repository.")
finally:
    if process is not None and process.poll() is None:
        stop()
    log.close()
