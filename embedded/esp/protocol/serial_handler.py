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

_MODULE   = "serial_handler"
_MAX_LINE = 1024  # max chars per line to avoid memory exhaustion

_poll        = None
_line_buffer = ""  # incomplete line across poll() calls

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


def _readline_nonblock():
    """
    Read a line from stdin without blocking.
    MicroPython select.poll + sys.stdin: readline()/read() block even when
    poll() indicates data is ready (see micropython/micropython#16550).
    Workaround: read(1) per character; buffer incomplete lines across calls.
    Returns complete line (without \\n) or None.
    """
    global _line_buffer
    while True:
        events = _poll.poll(0)
        if not events:
            return None
        c = sys.stdin.read(1)
        if not c:
            result = _line_buffer
            _line_buffer = ""
            return result if result else None
        if c == "\n" or c == "\r":
            result = _line_buffer
            _line_buffer = ""
            return result
        _line_buffer += c
        if len(_line_buffer) > _MAX_LINE:
            _line_buffer = ""
            return None


async def poll():
    """
    Non-blocking check for incoming serial data.
    If a complete line is available, parses and dispatches the command.
    Uses read(1) loop instead of readline() due to MicroPython poll+stdin limitation.
    Must be awaited in the main loop on every cycle.
    """
    if _poll is None:
        log_err(_MODULE, "poll", "not initialized — call init() first")
        return

    # Poll first — avoid building line when no data
    events = _poll.poll(0)
    if not events:
        return

    raw = _readline_nonblock()
    if not raw:
        return

    await _handle(raw)


async def _handle(raw):
    """Parse a raw string line, dispatch to the correct handler, write response.
    Regra de ouro: o único dado saindo pela serial deve ser o JSON de resposta.
    Sem log/print aqui — interferem no parse do host."""
    try:
        line = raw.strip() if isinstance(raw, str) else raw.decode().strip()
    except Exception:
        _respond({"ok": False, "error": "Decode error"})
        return

    try:
        msg = json.loads(line)
    except ValueError:
        _respond({"ok": False, "error": "Invalid JSON"})
        return

    cmd  = msg.get("cmd")
    data = msg.get("data")

    if not cmd:
        _respond({"ok": False, "error": "Missing cmd field"})
        return

    handler = _HANDLERS.get(cmd)
    if handler is None:
        _respond({"ok": False, "error": "Unknown command: {}".format(cmd)})
        return

    try:
        if cmd in ASYNC_COMMANDS:
            result = await handler(data)
        else:
            result = handler(data)
    except Exception:
        _respond({"ok": False, "error": "Internal error"})
        return

    _respond(result)

    if cmd == "reboot":
        import machine
        machine.reset()


def _respond(response):
    """Serialize and send JSON to host.
    Regra do protocolo: cada mensagem JSON termina com \\n.
    flush() garante envio imediato (MicroPython print nao suporta flush=True).
    """
    try:
        import time
        time.sleep_ms(50)  # let USB rx settle before tx
        sys.stdout.write(json.dumps(response) + "\n")
        sys.stdout.flush()
    except Exception as e:
        log_err(_MODULE, "_respond", "write error", e)
