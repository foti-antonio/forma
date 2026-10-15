#!/usr/bin/env python3
"""AT-SPI and focus-scoped X11 packaged workspace create/restart/close check; no injected IPC."""
import sys
import subprocess
import time
import gi

gi.require_version('Atspi', '2.0')
from gi.repository import Atspi

pid, phase, marker = int(sys.argv[1]), sys.argv[2], sys.argv[3]


def walk(node, depth=0):
    if node is None or depth > 40:
        return
    yield node
    for i in range(node.get_child_count()):
        yield from walk(node.get_child_at_index(i), depth + 1)


def nodes():
    desktop = Atspi.get_desktop(0)
    for i in range(desktop.get_child_count()):
        app = desktop.get_child_at_index(i)
        # Root children are applications. Never operate on another app.
        if app and app.get_process_id() == pid:
            return list(walk(app))
    return []


def until(find, label):
    end = time.monotonic() + 40
    while time.monotonic() < end:
        result = find()
        if result:
            return result
        time.sleep(.25)
    raise RuntimeError('AT-SPI timeout: ' + label)


def named(name, role=None):
    return next((n for n in nodes() if n.get_name() == name and
                 (role is None or n.get_role_name() == role)), None)


if phase == 'create':
    button = until(lambda: named('Create workspace', 'push button') or named('Create workspace', 'button'), 'welcome button')
    assert Atspi.Action.do_action(button, 0)
    entry = until(lambda: next((n for n in nodes() if n.get_name() == 'Workspace name' and n.get_state_set().contains(Atspi.StateType.EDITABLE)), None), 'workspace input')
    # WebKit HTML inputs do not expose AT-SPI EditableText. Focus using the
    # accessibility API, then deliver ordinary X11 input only to this app.
    assert Atspi.Component.grab_focus(entry)
    focused = subprocess.check_output(['xdotool', 'getwindowfocus'], text=True).strip()
    owner = subprocess.check_output(['xdotool', 'getwindowpid', focused], text=True).strip()
    assert int(owner) == pid, 'Refusing keyboard input outside the isolated app'
    subprocess.run(['xdotool', 'type', '--clearmodifiers', marker], check=True)
    button = until(lambda: next((n for n in nodes() if n.get_name() == 'Create workspace' and n.get_role_name() in ('push button', 'button') and n.get_state_set().contains(Atspi.StateType.ENABLED)), None), 'submit')
    assert Atspi.Action.do_action(button, 0)
until(lambda: any(marker in (n.get_name() or '') for n in nodes()), 'persisted workspace')
until(lambda: named('Export backup'), 'loaded workspace')
# Close through the window manager, scoped to the app's own X11 PID.
# AT-SPI verifies rendered content; EWMH requests ordinary window close.
subprocess.run(['wmctrl', '-i', '-c', hex(int(subprocess.check_output(
    ['xdotool', 'search', '--onlyvisible', '--pid', str(pid)], text=True).splitlines()[0]))], check=True)
print('PASS: AT-SPI workspace ' + phase + ' and window close requested')
