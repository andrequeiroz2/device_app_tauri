"""
Logging utility for embedded firmware.
Provides structured print-based logging (MicroPython has no logging module).

Usage:
    from tool.log import log, log_err

    log("my_module", "my_function", "message")
    log_err("my_module", "my_function", "what failed")
    log_err("my_module", "my_function", "what failed", exception)
"""


def log(module, fn, msg):
    print("[{}] [{}] {}".format(module, fn, msg))


def log_err(module, fn, msg, error=None):
    if error is not None:
        print("[{}] [{}] ERROR: {} | exception={}".format(module, fn, msg, error))
    else:
        print("[{}] [{}] ERROR: {}".format(module, fn, msg))
