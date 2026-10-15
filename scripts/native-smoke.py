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
root = Path(tempfile.mkdtemp(prefix="forma-m03-smoke-"))
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


print(f"Native smoke artifacts: {root}", flush=True)
try:
    launch()
    tabs(3)
    keys("-k", "Return")
    visible("Workspace name")
    text("smoke studio")
    keys("-k", "Return")
    until(lambda: rows("SELECT name FROM workspaces") == [("smoke studio",)], "workspace creation")
    visible("smoke studio")
    tabs(1, backwards=True)
    keys("-k", "Return")
    visible("Project name")
    text("smoke engine")
    keys("-k", "Return")
    until(lambda: rows("SELECT name FROM projects") == [("smoke engine",)], "project creation")
    time.sleep(0.5)
    tabs(5)
    keys("-k", "Return")
    visible("active issues")
    keys("n")
    visible("What needs to happen")
    text("parent task")
    save()
    until(lambda: rows("SELECT title FROM issues") == [("parent task",)], "issue creation")
    visible("parent task")
    # Search follows the List/Board view buttons and label management.
    search("")
    tabs(3, backwards=True)
    keys("-k", "Return")
    visible("Label name")
    text("smoke label")
    keys("-k", "Return")
    until(lambda: rows("SELECT name FROM labels") == [("smoke label",)], "label creation")
    keys("-k", "Escape")
    time.sleep(0.4)
    search("parent task")
    tabs(14)
    keys("-k", "Return")
    visible("Edit issue")
    text(" edited")
    tabs(2)
    text("**verified markdown**")
    save()
    until(lambda: rows("SELECT title,description FROM issues") == [("parent task edited", "**verified markdown**")], "issue editing")
    time.sleep(0.4)
    # The existing search still matches the edited parent. Open its child action.
    keys("/")
    tabs(14)
    keys("-k", "Return")
    visible("Edit issue")
    # WebKitGTK date controls have separate month/day/year tab stops.
    tabs(16)
    keys("-k", "Return")
    visible("What needs to happen")
    text("child task")
    tabs(11)
    keys("-k", "space")
    save()
    until(lambda: len(rows("SELECT id FROM issues WHERE parent_id IS NOT NULL")) == 1, "subtask creation")
    assert len(rows("SELECT * FROM issue_labels")) == 1
    assert rows("SELECT number FROM issues ORDER BY number") == [(1,), (2,)]
    # Return from the editor to the project, clear search, and select Board.
    keys("/")
    keys("-M", "ctrl", "-k", "a", "-m", "ctrl")
    keys("-k", "BackSpace")
    tabs(1, backwards=True)
    keys("-k", "Return")
    visible("Each column shows")
    until(lambda: rows("SELECT view FROM project_views") == [("board",)], "persistent Board selection")
    # Board button retains focus. Tab reaches search; clear the previous filter.
    tabs(1)
    keys("-M", "ctrl", "-k", "a", "-m", "ctrl")
    keys("-k", "BackSpace")
    time.sleep(0.8)
    # Manual sort is disabled on Board, so it is skipped in tab order.
    # Search -> filters (12 tab stops) -> first drag handle -> first card.
    tabs(14)
    keys("-M", "alt", "-k", "Right", "-m", "alt")
    until(lambda: rows("SELECT status FROM issues WHERE number=1") == [("in_progress",)], "keyboard move to In Progress")
    screenshot("keyboard-board")
    stop()
    launch()
    tabs(5)
    keys("-k", "Return")
    visible("child task")
    visible("Each column shows")
    assert rows("SELECT status FROM issues WHERE number=1") == [("in_progress",)]
    screenshot("restarted-board")
    keys("/")
    tabs(2, backwards=True)
    keys("-k", "Return")
    screenshot("restarted-list")
    search("child task")
    tabs(14)
    keys("-k", "Return")
    visible("Edit issue")
    tabs(11)
    visible("smoke label")
    screenshot("restarted-child")
    assert rows("SELECT title FROM issues WHERE parent_id IS NOT NULL") == [("child task",)]
    assert rows("PRAGMA integrity_check") == [("ok",)]
    assert rows("PRAGMA foreign_key_check") == []
    assert rows("PRAGMA user_version") == [(3,)]
    if "--backup" in sys.argv:
        keys("-k", "Escape")
        time.sleep(0.5)
        keys("/")
        tabs(11, backwards=True)
        native_dialog = "Export Forma backup"
        backup_path = root / "native.forma-backup"
        # One virtual keymap must cover GTK's creation and its input.
        keys("-k", "Return", "-s", "1800", "-M", "ctrl", "-k", "a", "-m", "ctrl",
             "-d", "30", str(backup_path), "-k", "Return")
        until(lambda: backup_path.exists(), "native snapshot export")
        until(lambda: active().get("pid") == process.pid, "return from native export dialog")
        native_dialog = None
        screenshot("export-complete")
        # Add data after export so restore must demonstrably replace it.
        time.sleep(0.5)
        keys("n")
        visible("What needs to happen")
        text("discard after backup")
        save()
        until(lambda: rows("SELECT count(*) FROM issues") == [(3,)], "post-export issue")
        time.sleep(0.7)
        keys("/")
        tabs(10, backwards=True)
        native_dialog = "Select Forma backup"
        keys("-k", "Return", "-s", "1800", "-M", "ctrl", "-k", "l", "-m", "ctrl",
             "-M", "ctrl", "-k", "a", "-m", "ctrl", "-d", "30", str(backup_path), "-k", "Return", "-s", "1200", "-k", "Return")
        until(lambda: active().get("pid") == process.pid, "return from native restore dialog")
        native_dialog = None
        visible("Replace all local data")
        screenshot("restore-confirmation")
        tabs(2)
        keys("-k", "Return")
        until(lambda: bool(list(database.parent.glob("forma-recovery-*.forma-backup"))), "native recovery copy")
        until(lambda: rows("SELECT count(*) FROM issues") == [(2,)], "native restored data")
        visible("Data restored")
        screenshot("restore-complete")
        assert rows("PRAGMA integrity_check") == [("ok",)]
        assert_snapshot_matches(backup_path)
        recovery_path = next(database.parent.glob("forma-recovery-*.forma-backup"))
        with sqlite3.connect(f"file:{recovery_path}?mode=ro", uri=True) as recovery:
            assert recovery.execute("SELECT count(*) FROM issues").fetchone() == (3,)
            assert recovery.execute("PRAGMA integrity_check").fetchone() == ("ok",)
            assert recovery.execute("PRAGMA foreign_key_check").fetchall() == []
        print("PASS: native export and restore file dialogs, explicit confirmation, recovery snapshot and frontend refresh.")
    print("PASS: workspace/project/issue create, issue edit, label assignment, subtask, filters, native keyboard Board movement and restart of view/order/status; SQLite integrity and foreign keys verified.")
finally:
    if process is not None and process.poll() is None:
        stop()
    log.close()
