"""
File system utility for embedded firmware.
Provides reusable helpers for file existence checks.
"""

import os

from tool.log import log, log_err

_MODULE = "load"


def file_exists(path):
    """
    Check if a file exists at the given path.
    Returns True if file exists, False otherwise.
    """
    try:
        os.stat(path)
        return True
    except OSError as e:
        log_err(_MODULE, "file_exists", "file not found | path='{}'".format(path), e)
        return False
