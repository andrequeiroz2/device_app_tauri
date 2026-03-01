"""
Read utility for embedded firmware.
Provides generic JSON file reading.
"""

import json

from tool.log import log, log_err

_MODULE = "read"


def read_json(path):
    """
    Read and parse a JSON file at path.
    Returns parsed dict or None if file is missing or invalid.
    """
    try:
        with open(path, "r") as f:
            data = json.load(f)
        log(_MODULE, "read_json", "read path='{}'".format(path))
        return data
    except (OSError, ValueError) as e:
        log_err(_MODULE, "read_json", "failed to read or parse path='{}'".format(path), e)
        return None
