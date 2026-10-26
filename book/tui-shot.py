#!/usr/bin/env python3
"""Screens of `relic-scan tui` on the demo index, as plain text: a pty at 100x30, keys sent, the screen read back with pyte.
   uv run --quiet --with pyte python tui-shot.py        (writes captures/3N-tui-*.txt)"""
import os, pty, sys, time, fcntl, termios, struct, select, pathlib, pyte, subprocess
B = pathlib.Path(__file__).parent; C = B / "captures"
DEMO = "/tmp/relic-demo"
COLS, ROWS = 100, 30

def session(steps, name, shown):
    pid, fd = pty.fork()
    if pid == 0:
        os.environ.update(HOME=DEMO, TERM="xterm-256color", NO_COLOR="1", RELIC_OPS_HUB="off")
        os.execvp("relic-scan", ["relic-scan", "tui"])
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))
    screen = pyte.Screen(COLS, ROWS); stream = pyte.ByteStream(screen)
    def pump(t=0.6):
        end = time.time() + t
        while time.time() < end:
            r, _, _ = select.select([fd], [], [], 0.1)
            if r:
                try: stream.feed(os.read(fd, 65536))
                except OSError: return
    pump(1.0)
    for k, snap in steps:
        if k: os.write(fd, k.encode()); pump(0.7)
        if snap:
            txt = "\n".join(l.rstrip() for l in screen.display).rstrip()
            (C / f"{snap}.txt").write_text(f"$ {shown}\n{txt}\n(exit 0 · screen {COLS}x{ROWS})\n")
            print(snap, len(txt.splitlines()), "lines")
    os.write(fd, b"q"); pump(0.3)
    try: os.waitpid(pid, 0)
    except ChildProcessError: pass

session([(None, "30-tui-overview"), ("2", "31-tui-repos"), ("\r", "32-tui-repo-sessions"), ("\x7f", None), ("3", "33-tui-sessions"),
         ("/", None), ("starfish", None), ("\r", "34-tui-filter"), ("\x1b", None), ("\r", "35-tui-popup"), ("\x1b", None), ("4", "36-tui-jsonl"),
         ("6", "37-tui-ops")], "tui", "relic-scan tui")
