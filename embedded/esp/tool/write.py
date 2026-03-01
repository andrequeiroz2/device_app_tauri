"""
Write utility for embedded firmware.
Provides generic JSON file writing.
"""

import json

from tool.log import log, log_err

_MODULE = "write"


def write_json(path, data):
    """
    Serialize data as JSON and write to path.
    Returns True on success, False on OSError.
    """
    try:
        with open(path, "w") as f:
            json.dump(data, f)
        log(_MODULE, "write_json", "written path='{}'".format(path))
        return True
    except OSError as e:
        log_err(_MODULE, "write_json", "failed to write path='{}'".format(path), e)
        return False
