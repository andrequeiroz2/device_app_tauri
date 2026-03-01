"""
WiFi utility for embedded firmware.
Provides reusable helpers for wifi.json manipulation.
"""

from tool.log   import log, log_err
from tool.read  import read_json
from tool.write import write_json

_MODULE = "wifi"


def set_default(wifi_file, ssid):
    """
    Set default=True for the given ssid in wifi_file, False for all others.
    Only one entry can be default at a time.
    Returns True on success, False if ssid not found or write error.
    """
    config = read_json(wifi_file)
    if config is None or "wifi" not in config:
        log_err(_MODULE, "set_default", "could not read wifi file | path='{}'".format(wifi_file))
        return False

    updated = False
    for entry in config["wifi"]:
        if entry.get("ssid") == ssid:
            entry["default"] = True
            updated = True
        else:
            entry["default"] = False

    if not updated:
        log_err(_MODULE, "set_default", "ssid not found | ssid='{}'".format(ssid))
        return False

    log(_MODULE, "set_default", "default set to ssid='{}'".format(ssid))
    return write_json(wifi_file, config)
