#!/usr/bin/env python3
"""mudctl - persistent telnet session manager for tbaMUD/CircleMUD servers.

A background daemon owns the TCP socket and streams everything the MUD sends
into a session log. Short-lived CLI calls talk to the daemon over a unix
socket, so each `send` returns only the output that command produced.

  mudctl.py start [--host H] [--port P] [--user U] [--password P]
  mudctl.py send "look"
  mudctl.py read [--lines N] [--all]
  mudctl.py status
  mudctl.py stop

Telnet negotiation is answered with a blanket refusal (WONT/DONT), which is
what CircleMUD derivatives expect from a plain client.
"""

import argparse
import json
import os
import re
import selectors
import socket
import subprocess
import sys
import time

STATE_ROOT = os.path.expanduser("~/.mud-sessions")

# --- telnet protocol bytes ---
IAC, DONT, DO, WONT, WILL, SB, SE = 255, 254, 253, 252, 251, 250, 240

ANSI_RE = re.compile(rb"\x1b\[[0-9;?]*[ -/]*[@-~]")


def state_dir(session):
    return os.path.join(STATE_ROOT, session)


def paths(session):
    d = state_dir(session)
    return {
        "dir": d,
        "sock": os.path.join(d, "ctl.sock"),
        "log": os.path.join(d, "session.log"),
        "meta": os.path.join(d, "meta.json"),
        "daemon_log": os.path.join(d, "daemon.log"),
    }


def strip_output(data: bytes) -> bytes:
    """Remove telnet negotiation sequences and ANSI colour codes."""
    out = bytearray()
    i = 0
    n = len(data)
    while i < n:
        b = data[i]
        if b == IAC:
            if i + 1 >= n:
                break
            cmd = data[i + 1]
            if cmd in (DO, DONT, WILL, WONT):
                i += 3
                continue
            if cmd == SB:
                j = data.find(bytes([IAC, SE]), i)
                i = n if j == -1 else j + 2
                continue
            if cmd == IAC:
                out.append(IAC)
                i += 2
                continue
            i += 2
            continue
        out.append(b)
        i += 1
    return ANSI_RE.sub(b"", bytes(out)).replace(b"\r\n", b"\n").replace(b"\r", b"")


def negotiation_reply(data: bytes) -> bytes:
    """Refuse every telnet option the server offers or requests."""
    reply = bytearray()
    i = 0
    while i < len(data) - 2:
        if data[i] == IAC and data[i + 1] in (DO, DONT, WILL, WONT):
            cmd, opt = data[i + 1], data[i + 2]
            if cmd == DO:
                reply += bytes([IAC, WONT, opt])
            elif cmd == WILL:
                reply += bytes([IAC, DONT, opt])
            i += 3
            continue
        i += 1
    return bytes(reply)


# ---------------------------------------------------------------- daemon side

def run_daemon(session, host, port, user, password):
    p = paths(session)
    os.makedirs(p["dir"], exist_ok=True)
    for stale in (p["sock"],):
        if os.path.exists(stale):
            os.unlink(stale)

    log = open(p["log"], "ab", buffering=0)

    mud = socket.create_connection((host, port), timeout=10)
    mud.setblocking(False)

    ctl = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    ctl.bind(p["sock"])
    ctl.listen(8)
    ctl.setblocking(False)

    with open(p["meta"], "w") as f:
        json.dump(
            {
                "session": session,
                "host": host,
                "port": port,
                "user": user,
                "pid": os.getpid(),
                "started": time.time(),
            },
            f,
        )

    sel = selectors.DefaultSelector()
    sel.register(mud, selectors.EVENT_READ, "mud")
    sel.register(ctl, selectors.EVENT_READ, "ctl")

    login_steps = []
    if user:
        # tbaMUD: name -> password -> ENTER past MOTD -> menu choice 1
        login_steps = [
            (rb"[Nn]ame|[Nn]ame you wish", user),
            (rb"[Pp]assword", password),
            (rb"PRESS RETURN|press return|\[ Return", ""),
            (rb"Make a choice|^\s*1\)|Enter the game", "1"),
        ]
    pending = list(login_steps)
    tail = b""
    running = True

    def send_line(text):
        mud.sendall(text.encode("utf-8", "replace") + b"\r\n")

    while running:
        for key, _ in sel.select(timeout=0.5):
            if key.data == "mud":
                try:
                    chunk = mud.recv(65536)
                except BlockingIOError:
                    continue
                if not chunk:
                    log.write(b"\n[connection closed by server]\n")
                    running = False
                    break
                nego = negotiation_reply(chunk)
                if nego:
                    try:
                        mud.sendall(nego)
                    except OSError:
                        pass
                clean = strip_output(chunk)
                log.write(clean)
                tail = (tail + clean)[-4000:]
                # advance the login script as prompts appear
                while pending:
                    pattern, value = pending[0]
                    if re.search(pattern, tail, re.MULTILINE):
                        time.sleep(0.3)
                        send_line(value)
                        pending.pop(0)
                        tail = b""
                    else:
                        break
                # Reconnecting a linkdead character skips the MOTD and menu.
                # Once the game prompt shows, disarm the rest of the login
                # script so it can never inject stray input mid-session.
                if pending and len(pending) <= 2 and re.search(
                    rb"\d+H\s+\d+M\s+\d+V|Welcome to the land", tail
                ):
                    pending.clear()

            elif key.data == "ctl":
                conn, _ = ctl.accept()
                conn.settimeout(5)
                try:
                    raw = conn.recv(65536).decode("utf-8", "replace").strip()
                    req = json.loads(raw) if raw else {}
                    op = req.get("op")
                    if op == "send":
                        send_line(req.get("data", ""))
                        resp = {"ok": True}
                    elif op == "raw":
                        mud.sendall(req.get("data", "").encode())
                        resp = {"ok": True}
                    elif op == "status":
                        resp = {
                            "ok": True,
                            "host": host,
                            "port": port,
                            "user": user,
                            "login_pending": len(pending),
                            "log_size": os.path.getsize(p["log"]),
                        }
                    elif op == "stop":
                        try:
                            send_line("quit")
                        except OSError:
                            pass
                        resp = {"ok": True}
                        running = False
                    else:
                        resp = {"ok": False, "error": f"unknown op {op!r}"}
                    conn.sendall((json.dumps(resp) + "\n").encode())
                except Exception as exc:  # keep the daemon alive on bad input
                    try:
                        conn.sendall(
                            (json.dumps({"ok": False, "error": str(exc)}) + "\n").encode()
                        )
                    except OSError:
                        pass
                finally:
                    conn.close()

    sel.close()
    try:
        mud.close()
    finally:
        ctl.close()
        if os.path.exists(p["sock"]):
            os.unlink(p["sock"])
        log.close()


# ------------------------------------------------------------------ CLI side

def call(session, req, timeout=5):
    p = paths(session)
    if not os.path.exists(p["sock"]):
        raise SystemExit(
            f"no live session {session!r}. Run: mudctl.py start --session {session}"
        )
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.settimeout(timeout)
    try:
        s.connect(p["sock"])
    except (ConnectionRefusedError, FileNotFoundError):
        raise SystemExit(f"session {session!r} socket is stale. Run `stop` then `start`.")
    s.sendall((json.dumps(req) + "\n").encode())
    data = s.recv(65536).decode("utf-8", "replace")
    s.close()
    return json.loads(data)


def read_from(session, offset):
    p = paths(session)
    with open(p["log"], "rb") as f:
        f.seek(offset)
        return f.read().decode("utf-8", "replace")


def log_size(session):
    try:
        return os.path.getsize(paths(session)["log"])
    except FileNotFoundError:
        return 0


def wait_for_output(session, offset, wait, quiet):
    """Collect output until the stream goes quiet for `quiet` seconds."""
    deadline = time.time() + wait
    last_size = offset
    last_change = time.time()
    while time.time() < deadline:
        time.sleep(0.1)
        size = log_size(session)
        if size != last_size:
            last_size = size
            last_change = time.time()
        elif size > offset and time.time() - last_change >= quiet:
            break
    return read_from(session, offset)


def cmd_start(args):
    p = paths(args.session)
    os.makedirs(p["dir"], exist_ok=True)

    if os.path.exists(p["sock"]):
        try:
            call(args.session, {"op": "status"}, timeout=2)
            print(f"session {args.session!r} is already running")
            return
        except SystemExit:
            os.unlink(p["sock"])

    open(p["log"], "wb").close()  # fresh log per session
    dlog = open(p["daemon_log"], "ab")
    subprocess.Popen(
        [
            sys.executable, os.path.abspath(__file__),
            "--session", args.session, "_daemon",
            "--host", args.host, "--port", str(args.port),
            "--user", args.user, "--password", args.password,
        ],
        stdout=dlog, stderr=dlog, stdin=subprocess.DEVNULL,
        start_new_session=True,
    )

    for _ in range(60):
        time.sleep(0.25)
        if os.path.exists(p["sock"]):
            break
    else:
        raise SystemExit(f"daemon failed to start; see {p['daemon_log']}")

    print(wait_for_output(args.session, 0, wait=args.wait, quiet=args.quiet))


def cmd_send(args):
    offset = log_size(args.session)
    call(args.session, {"op": "send", "data": args.command})
    print(wait_for_output(args.session, offset, wait=args.wait, quiet=args.quiet))


def cmd_read(args):
    text = read_from(args.session, 0)
    if not args.all:
        lines = text.splitlines()
        text = "\n".join(lines[-args.lines:])
    print(text)


def cmd_status(args):
    print(json.dumps(call(args.session, {"op": "status"}), indent=2))


def cmd_stop(args):
    try:
        call(args.session, {"op": "stop"})
        print(f"session {args.session!r} stopped")
    except SystemExit as exc:
        print(exc)
    p = paths(args.session)
    if os.path.exists(p["sock"]):
        os.unlink(p["sock"])


def main():
    ap = argparse.ArgumentParser(description="tbaMUD/CircleMUD telnet session manager")
    ap.add_argument("--session", default="default", help="session name")
    sub = ap.add_subparsers(dest="cmd", required=True)

    def timing(sp, wait, quiet):
        sp.add_argument("--wait", type=float, default=wait,
                        help="max seconds to collect output")
        sp.add_argument("--quiet", type=float, default=quiet,
                        help="seconds of silence that means output is done")

    sp = sub.add_parser("start", help="connect and log in")
    sp.add_argument("--host", default=os.environ.get("MUD_HOST", "localhost"))
    sp.add_argument("--port", type=int, default=int(os.environ.get("MUD_PORT", 4000)))
    sp.add_argument("--user", default=os.environ.get("MUD_USER", ""))
    sp.add_argument("--password", default=os.environ.get("MUD_PASSWORD", ""))
    timing(sp, 12.0, 1.5)
    sp.set_defaults(func=cmd_start)

    sp = sub.add_parser("send", help="send one command, print its output")
    sp.add_argument("command")
    timing(sp, 4.0, 0.5)
    sp.set_defaults(func=cmd_send)

    sp = sub.add_parser("read", help="replay the session log")
    sp.add_argument("--lines", type=int, default=60)
    sp.add_argument("--all", action="store_true")
    sp.set_defaults(func=cmd_read)

    sub.add_parser("status", help="show session info").set_defaults(func=cmd_status)
    sub.add_parser("stop", help="quit and disconnect").set_defaults(func=cmd_stop)

    sp = sub.add_parser("_daemon", help=argparse.SUPPRESS)
    sp.add_argument("--host", required=True)
    sp.add_argument("--port", type=int, required=True)
    sp.add_argument("--user", default="")
    sp.add_argument("--password", default="")
    sp.set_defaults(func=lambda a: run_daemon(a.session, a.host, a.port, a.user, a.password))

    args = ap.parse_args()
    args.func(args)


if __name__ == "__main__":
    main()
