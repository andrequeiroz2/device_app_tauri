"""
WiFi configuration module.
Manages known WiFi networks in /wifi.json and handles connection on boot.
"""

import network
import uasyncio as asyncio

from tool.load  import file_exists
from tool.log   import log, log_err
from tool.read  import read_json
from tool.wifi  import set_default
from tool.write import write_json

WIFI_FILE = "/wifi.json"
_MODULE   = "config_wifi"

_INIT_ENTRY = {"ssid": "", "pass": "", "default": False}


# --- Private helpers ---

def _is_entry_valid(entry):
    """Return True if entry has a non-empty ssid string."""
    ssid = entry.get("ssid", "")
    return isinstance(ssid, str) and ssid.strip() != ""


def remove_wifi(ssid):
    """
    Remove the entry with the given ssid from wifi.json.
    Used for rollback when adoption fails after add_wifi.
    Returns True on success, False otherwise.
    """
    config = read_json(WIFI_FILE)
    if config is None or "wifi" not in config:
        log_err(_MODULE, "_remove_wifi", "could not read wifi.json for rollback")
        return False

    original_len = len(config["wifi"])
    config["wifi"] = [e for e in config["wifi"] if e.get("ssid") != ssid]

    if len(config["wifi"]) == original_len:
        log_err(_MODULE, "_remove_wifi", "ssid not found for rollback | ssid='{}'".format(ssid))
        return False

    log(_MODULE, "_remove_wifi", "removed ssid='{}' (rollback)".format(ssid))
    return write_json(WIFI_FILE, config)


# --- Public API ---

def write_init_config():
    """
    Ensure /wifi.json exists with a valid structure.
    Creates the file with an empty entry if missing or corrupted.
    Returns True on success, False on write error.
    """
    if not file_exists(WIFI_FILE):
        log(_MODULE, "write_init_config", "wifi.json not found, creating")
        return write_json(WIFI_FILE, {"wifi": [_INIT_ENTRY]})

    config = read_json(WIFI_FILE)
    if config is None:
        log_err(_MODULE, "write_init_config", "wifi.json exists but is unreadable, recreating")
        return write_json(WIFI_FILE, {"wifi": [_INIT_ENTRY]})

    if "wifi" in config and isinstance(config["wifi"], list):
        log(_MODULE, "write_init_config", "wifi.json already initialized")
        return True

    log(_MODULE, "write_init_config", "wifi.json has invalid structure, recreating")
    return write_json(WIFI_FILE, {"wifi": [_INIT_ENTRY]})


def add_wifi(ssid, password):
    """
    Upsert a WiFi entry: insert if ssid is new, update pass if it already exists.
    Does not change the default field — that is set exclusively by set_default().
    Returns True on success, False on validation or write error.
    """
    if not isinstance(ssid, str) or not ssid.strip():
        log_err(_MODULE, "add_wifi", "ssid must be a non-empty string | type={}".format(type(ssid)))
        return False

    if not isinstance(password, str):
        log_err(_MODULE, "add_wifi", "password must be a string | type={}".format(type(password)))
        return False

    config = read_json(WIFI_FILE)
    if config is None or "wifi" not in config:
        log(_MODULE, "add_wifi", "wifi.json missing or invalid, initializing")
        config = {"wifi": []}

    found = False
    for entry in config["wifi"]:
        if entry.get("ssid") == ssid:
            entry["pass"] = password
            found = True
            log(_MODULE, "add_wifi", "updated existing entry ssid='{}'".format(ssid))
            break

    if not found:
        config["wifi"].append({"ssid": ssid, "pass": password, "default": False})
        log(_MODULE, "add_wifi", "added new entry ssid='{}'".format(ssid))

    return write_json(WIFI_FILE, config)



async def connect(ssid, password, timeout=10):
    """
    Connect to the given WiFi network.
    On success, calls set_default(ssid) and returns True.
    On timeout, disconnects and returns False.
    """
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)

    if wlan.isconnected():
        current = wlan.config("essid")
        if current == ssid:
            log(_MODULE, "connect", "already connected to ssid='{}'".format(ssid))
            if not set_default(WIFI_FILE, ssid):
                log_err(_MODULE, "connect", "connected but failed to set default | ssid='{}'".format(ssid))
            return True
        log(_MODULE, "connect", "disconnecting from ssid='{}' to connect to '{}'".format(current, ssid))
        wlan.disconnect()
        await asyncio.sleep(2)

    log(_MODULE, "connect", "connecting to ssid='{}'".format(ssid))
    wlan.config(reconnects=1)  # prevent infinite retry — our loop controls timeout
    wlan.connect(ssid, password)

    for _ in range(timeout * 2):
        if wlan.isconnected():
            ip = wlan.ifconfig()[0]
            log(_MODULE, "connect", "connected ssid='{}' ip={}".format(ssid, ip))
            if not set_default(WIFI_FILE, ssid):
                log_err(_MODULE, "connect", "connected but failed to set default | ssid='{}'".format(ssid))
            return True
        await asyncio.sleep(0.5)

    wlan.disconnect()
    log_err(_MODULE, "connect", "connection timeout | ssid='{}'".format(ssid))
    return False


async def connect_from_config():
    """
    Try to connect using entries in wifi.json.
    Attempts default=True entry first, then remaining entries in order.
    Skips entries with empty ssid.
    Returns True on first successful connection, False if all fail.
    """
    config = read_json(WIFI_FILE)
    if config is None or "wifi" not in config:
        log_err(_MODULE, "connect_from_config", "could not read wifi.json")
        return False

    wifi_list = config["wifi"]
    if not wifi_list:
        log_err(_MODULE, "connect_from_config", "wifi list is empty")
        return False

    default_net = None
    other_nets  = []
    for e in wifi_list:
        if e.get("default") is True:
            default_net = e
        else:
            other_nets.append(e)
    networks_to_try = ([default_net] + other_nets) if default_net else other_nets

    for entry in networks_to_try:
        if not _is_entry_valid(entry):
            log(_MODULE, "connect_from_config", "skipping empty entry")
            continue

        ssid     = entry["ssid"]
        password = entry.get("pass", "")
        log(_MODULE, "connect_from_config", "trying ssid='{}'".format(ssid))

        if await connect(ssid, password):
            log(_MODULE, "connect_from_config", "connected via ssid='{}'".format(ssid))
            return True

    log_err(_MODULE, "connect_from_config", "could not connect to any network")
    return False


def get_status():
    """
    Return current WiFi connection status.
    Returns dict with connected, ssid, ip, netmask, gateway, dns, rssi.
    """
    status = {
        "connected": False,
        "ssid":      None,
        "ip":        None,
        "netmask":   None,
        "gateway":   None,
        "dns":       None,
        "rssi":      None,
    }

    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)

    if not wlan.isconnected():
        log_err(_MODULE, "get_status", "not connected")
        return status

    ip, netmask, gateway, dns = wlan.ifconfig()
    rssi = None
    try:
        rssi = wlan.status("rssi")
    except Exception:
        pass

    status["connected"] = True
    status["ssid"]      = wlan.config("essid")
    status["ip"]        = ip
    status["netmask"]   = netmask
    status["gateway"]   = gateway
    status["dns"]       = dns
    status["rssi"]      = rssi

    log(_MODULE, "get_status", "connected ssid='{}' ip={}".format(status["ssid"], ip))
    return status
