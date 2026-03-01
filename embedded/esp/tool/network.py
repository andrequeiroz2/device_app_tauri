"""
Network utility for embedded firmware.
Provides hardware-level network helpers (MAC address, WiFi).
"""

import network

from tool.log import log, log_err

_MODULE = "network"


def get_mac_address():
    """
    Get MAC address from WLAN interface.
    Returns formatted string e.g. "3C:71:BF:4D:DB:0C" or None on error.
    """
    try:
        wlan = network.WLAN(network.STA_IF)
        if not wlan.active():
            wlan.active(True)
        mac = wlan.config("mac")
        result = ":".join("{:02X}".format(b) for b in mac)
        log(_MODULE, "get_mac_address", "mac={}".format(result))
        return result
    except (OSError, AttributeError) as e:
        log_err(_MODULE, "get_mac_address", "failed to read MAC address", e)
        return None
