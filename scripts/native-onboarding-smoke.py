#!/usr/bin/env python3
"""Opt-in Linux/Hyprland M08 UI smoke. Uses isolated data and a compiled fake CLI.

Requires a desktop build, PyGObject/AT-SPI, wtype, grim, Git and rustc. All input
is scoped to the test app/native folder picker. No injected IPC or SQL writes.
Keep Forma focused. Screenshots and logs stay in the printed temporary directory.
"""
import json
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import time
import gi

gi.require_version('Atspi', '2.0')
from gi.repository import Atspi, GLib

binary = Path(sys.argv[1] if len(sys.argv) > 1 else 'src-tauri/target/debug/forma').resolve()
root = Path(tempfile.mkdtemp(prefix='forma-m08-native-'))
print('Artifacts:', root, flush=True)
fixture_dir = root / 'provider'
fixture_dir.mkdir()
fixture = fixture_dir / 'codex'
subprocess.run(['rustc', '--edition=2021', 'scripts/agent-fixture.rs', '-o', str(fixture)], check=True)
repo = root / 'repository'
subprocess.run(['git', 'init', '-b', 'main', str(repo)], check=True, stdout=subprocess.DEVNULL)
(repo / 'tracked').write_text('original\n')
subprocess.run(['git', '-C', str(repo), 'add', 'tracked'], check=True)
subprocess.run(['git', '-C', str(repo), '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', 'commit', '-m', 'Initial'], check=True, stdout=subprocess.DEVNULL)
env = dict(os.environ, XDG_DATA_HOME=str(root / 'data'), XDG_CONFIG_HOME=str(root / 'config'), XDG_CACHE_HOME=str(root / 'cache'), CODEX_INSTALL_DIR=str(fixture_dir), GTK_A11Y='1')
db = root / 'data/dev.forma.desktop/forma.sqlite3'
log = (root / 'native.log').open('w')
process = subprocess.Popen([str(binary)], env=env, stdout=log, stderr=log)
portal = False


def until(action, label, timeout=35):
    end = time.monotonic() + timeout
    while time.monotonic() < end:
        assert process.poll() is None, f'App exited: {process.returncode}; {root}'
        try:
            value = action()
            if value:
                return value
        except GLib.GError:
            pass
        time.sleep(.2)
    raise RuntimeError(f'Timeout: {label}; {root}')


def active():
    return json.loads(subprocess.check_output(['hyprctl', '-j', 'activewindow']))


def focused():
    w = active()
    return w.get('pid') == process.pid or (portal and w.get('title') == 'Connect Git repository' and w.get('class') in ('xdg-desktop-portal-gtk', 'org.freedesktop.impl.portal.desktop.gtk'))


def keys(*args):
    assert focused(), 'Refusing keyboard input outside the test app'
    subprocess.run(['wtype', '-s', '100', *args, '-s', '100'], check=True)
    time.sleep(.2)


def walk(node, depth=0):
    if node is None or depth > 40:
        return
    yield node
    for i in range(node.get_child_count()):
        yield from walk(node.get_child_at_index(i), depth + 1)


def nodes(native=False):
    context = GLib.MainContext.default()
    while context.pending():
        context.iteration(False)
    desktop = Atspi.get_desktop(0)
    for i in range(desktop.get_child_count()):
        app = desktop.get_child_at_index(i)
        if app and (app.get_name() == 'xdg-desktop-portal-gtk' if native else app.get_process_id() == process.pid):
            return list(walk(app))
    return []


def control(name, roles=None, partial=False):
    return until(lambda: next((n for n in nodes() if (n.get_name() == name or (partial and name in n.get_name())) and (not roles or n.get_role_name() in roles) and n.get_state_set().contains(Atspi.StateType.ENABLED)), None), 'control: ' + name)


def click(name, partial=False):
    assert focused(), 'Refusing accessible input outside test app'
    node = control(name, ('button', 'push button', 'toggle button', 'check box', 'unknown'), partial)
    if not Atspi.Action.do_action(node, 0):
        assert Atspi.Component.grab_focus(node)
        keys('-k', 'Return')
    time.sleep(.25)


def fill(name, value):
    node = control(name, ('entry', 'text'))
    assert Atspi.Component.grab_focus(node)
    keys('-d', '20', value)


def rows(sql):
    if not db.exists():
        return []
    with sqlite3.connect(f'file:{db}?mode=ro', uri=True) as connection:
        return connection.execute(sql).fetchall()


def screenshot(name):
    assert focused()
    w = active()
    subprocess.run(['grim', '-g', f"{w['at'][0]},{w['at'][1]} {w['size'][0]}x{w['size'][1]}", str(root / (name + '.png'))], check=True)


def permissions():
    click('Apply recommended configuration')
    click('I understand these permissions', partial=True)
    click('Check readiness')


try:
    until(focused, 'focused Forma window')
    click('AI agents & runs')
    click('Agent settings')
    click('Set up AI')
    click('Find installation')
    screenshot('01-discovery')
    assert rows("SELECT trusted FROM agent_settings WHERE provider='codex'") == [(0,)]
    assert rows('SELECT count(*) FROM agent_runs') == [(0,)]
    click('Authorize this installation: ' + str(fixture.resolve()))
    until(lambda: rows("SELECT trusted FROM agent_settings WHERE provider='codex'") == [(1,)], 'authorized discovered fixture')
    click('Continue to sign-in')
    click('Open Codex login')
    screenshot('02-explicit-login')
    click('Cancel login')
    click('Check authentication')
    click('Continue')
    permissions()
    click('Finish setup')
    control('Set up AI', ('button', 'push button'))
    click('Close dialog')
    click('Create workspace')
    fill('Workspace name', 'Onboarding smoke')
    keys('-k', 'Return')
    until(lambda: rows('SELECT count(*) FROM workspaces') == [(1,)], 'workspace')
    click('New project')
    fill('Project name', 'Onboarding engine')
    keys('-k', 'Return')
    until(lambda: rows('SELECT count(*) FROM projects') == [(1,)], 'project')
    click('Onboarding engine')
    click('New issue')
    fill('Title', 'First approved task')
    keys('-M', 'ctrl', '-k', 'Return', '-m', 'ctrl')
    until(lambda: rows('SELECT count(*) FROM issues') == [(1,)], 'saved issue')
    click('First approved task', partial=True)
    click('Run with AI')
    click('Find installation')
    click('Continue to sign-in')
    click('Continue')
    portal = True
    # Focus the real native picker action before sending Return.
    node = control('Select Git folder…', ('button', 'push button'))
    assert Atspi.Component.grab_focus(node)
    keys('-k', 'Return')
    until(lambda: active().get('title') == 'Connect Git repository', 'native folder dialog')
    keys('/')
    entry = until(lambda: next((n for n in nodes(True) if n.get_role_name() == 'text' and n.get_state_set().contains(Atspi.StateType.SHOWING)), None), 'native location entry')
    Atspi.EditableText.set_text_contents(entry, str(repo))
    Atspi.Action.do_action(entry, 0)
    def accept_folder():
        if active().get('pid') == process.pid:
            return True
        assert focused()
        node = next((n for n in nodes(True) if n.get_name() in ('Select', 'Open') and n.get_role_name() == 'button' and n.get_state_set().contains(Atspi.StateType.ENABLED)), None)
        return node is not None and Atspi.Action.do_action(node, 0)
    until(accept_folder, 'accept folder')
    until(lambda: active().get('pid') == process.pid, 'return to Forma')
    portal = False
    click('Continue to permissions')
    permissions()
    screenshot('03-ready')
    assert rows('SELECT count(*) FROM agent_runs') == [(0,)]
    assert not (root / 'data/dev.forma.desktop/agent-worktrees').exists()
    click('Preview first task')
    prompt = control('Task prompt (saved issue)', ('text', 'entry'))
    value = Atspi.Text.get_text(prompt, 0, -1)
    assert 'First approved task' in value and 'UNTRUSTED TASK DATA' in value
    screenshot('04-exact-approval')
    assert rows('SELECT count(*) FROM agent_runs') == [(0,)]
    click('Approve & run once')
    until(lambda: rows("SELECT count(*) FROM agent_runs WHERE status='Completed'") == [(1,)], 'approved task completed')
    assert not (repo / 'agent-result.txt').exists()
    worktree = Path(rows('SELECT worktree FROM agent_runs')[0][0])
    assert (worktree / 'agent-result.txt').read_text() == 'simulated provider change\n'
    click('Inspect result')
    screenshot('05-result-review')
    click('Accept for manual integration')
    until(lambda: rows("SELECT count(*) FROM agent_runs WHERE review='accepted'") == [(1,)], 'review persisted')
    print('PASS: native discovered authorization without path entry, explicit login/cancel, permissions, native repository connection, exact prompt, final approval, coordinator execution and review.', flush=True)
finally:
    if process.poll() is None:
        process.terminate()
    process.wait(timeout=15)
    log.close()
    if db.exists():
        with sqlite3.connect(f'file:{db}?mode=ro', uri=True) as connection:
            assert connection.execute('PRAGMA integrity_check').fetchall() == [('ok',)]
            assert connection.execute('PRAGMA foreign_key_check').fetchall() == []
    print('App exit:', process.returncode, '; inspect native.log for subprocess shutdown warnings.', flush=True)
