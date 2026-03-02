"""
Serial handler for embedded firmware.
Reads JSON commands from UART0 (USB-CDC), dispatches to command handlers,
writes JSON responses. Non-blocking via select.poll.

Uses sys.stdin/sys.stdout instead of UART(0) because UART0 is already
initialized by MicroPython's REPL and cannot be re-created.
"""

import json
import select
import sys

from protocol.commands import (
    handle_ping,
    handle_get_info,
    handle_get_config,
    handle_set_config,
    handle_reboot,
    ASYNC_COMMANDS,
)
from tool.log import log, log_err

_MODULE = "serial_handler"

_poll = None

_HANDLERS = {
    "ping":       handle_ping,
    "get_info":   handle_get_info,
    "get_config": handle_get_config,
    "set_config": handle_set_config,
    "reboot":     handle_reboot,
}


def init():
    """
    Register sys.stdin with select.poll for non-blocking reads.
    sys.stdin is already connected to UART0 (USB-CDC) by MicroPython.
    Must be called once before poll().
    """
    global _poll
    _poll = select.poll()
    _poll.register(sys.stdin, select.POLLIN)
    log(_MODULE, "init", "serial handler ready via sys.stdin")


async def poll():
    """
    Non-blocking check for incoming serial data.
    If a line is available, parses and dispatches the command.
    Must be awaited in the main loop on every cycle.
    """
    if _poll is None:
        log_err(_MODULE, "poll", "not initialized — call init() first")
        return

    events = _poll.poll(0)
    if not events:
        return

    raw = sys.stdin.readline()
    if not raw:
        return

    await _handle(raw)


async def _handle(raw):
    """Parse a raw bytes line, dispatch to the correct handler, write response."""
    try:
        line = raw.decode().strip()
    except Exception as e:
        log_err(_MODULE, "_handle", "decode error", e)
        _respond({"ok": False, "error": "Decode error"})
        return

    log(_MODULE, "_handle", "received: {}".format(line))

    try:
        msg = json.loads(line)
    except ValueError as e:
        log_err(_MODULE, "_handle", "JSON parse error | raw='{}'".format(line), e)
        _respond({"ok": False, "error": "Invalid JSON"})
        return

    cmd  = msg.get("cmd")
    data = msg.get("data")

    if not cmd:
        log_err(_MODULE, "_handle", "missing 'cmd' field")
        _respond({"ok": False, "error": "Missing cmd field"})
        return

    handler = _HANDLERS.get(cmd)
    if handler is None:
        log_err(_MODULE, "_handle", "unknown cmd='{}'".format(cmd))
        _respond({"ok": False, "error": "Unknown command: {}".format(cmd)})
        return

    log(_MODULE, "_handle", "dispatching cmd='{}'".format(cmd))

    try:
        if cmd in ASYNC_COMMANDS:
            result = await handler(data)
        else:
            result = handler(data)
    except Exception as e:
        log_err(_MODULE, "_handle", "handler exception cmd='{}'".format(cmd), e)
        _respond({"ok": False, "error": "Internal error"})
        return

    _respond(result)

    if cmd == "reboot":
        import machine
        machine.reset()


def _respond(response):
    """Serialize response dict as JSON and write to sys.stdout with newline terminator."""
    try:
        line = json.dumps(response) + "\n"
        sys.stdout.write(line)
        log(_MODULE, "_respond", "sent: {}".format(line.strip()))
    except Exception as e:
        log_err(_MODULE, "_respond", "write error", e)
