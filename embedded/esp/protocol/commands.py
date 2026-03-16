"""
Protocol command handlers for serial communication.
Each handler receives the payload data dict and returns a response dict.
set_config is async — must be awaited by the dispatcher.
"""

import machine

from config.config    import read_config, adopt_device
from tool.log         import log, log_err
from tool.validate    import validate_adoption_data
from wifi.config_wifi import add_wifi, connect, remove_wifi

_MODULE          = "commands"
FIRMWARE_VERSION = "1.0.0"

# Commands that require await in the dispatcher
ASYNC_COMMANDS = ("set_config",)

# Set to True after successful adoption — checked by main loop to avoid file reads
_adopted = False


def is_adopted():
    """Return True if set_config completed successfully in this session."""
    return _adopted


def handle_ping(data):
    """Return firmware version."""
    return {"ok": True, "version": FIRMWARE_VERSION}


def handle_get_info(data):
    """Return device identity fields and firmware version.
    Includes parameter_ranges (sensor) / command_spec (actuator)
    When adopted, also returns user_uuid for ownership check."""
    config = read_config()
    if config is None:
        return {"ok": False, "error": "Could not read device config"}

    out = {
        "ok":               True,
        "adopted_status":   config.get("adopted_status"),
        "device_type":      config.get("device_type"),
        "sensor_type":      config.get("sensor_type"),
        "actuator_type":    config.get("actuator_type"),
        "boarder_type":     config.get("boarder_type"),
        "mac_address":      config.get("mac_address"),
        "device_scale":     config.get("device_scale"),
        "parameter_ranges": config.get("parameter_ranges"),
        "command_spec":     config.get("command_spec"),
        "firmware_version": FIRMWARE_VERSION,
    }
    if config.get("adopted_status") == 1:
        out["user_uuid"] = config.get("user_uuid", "")
    return out


def handle_get_config(data):
    """Return full config.json content."""
    config = read_config()
    if config is None:
        return {"ok": False, "error": "Could not read device config"}

    response = dict(config)
    response["ok"] = True
    return response


async def handle_set_config(data):
    """
    Atomic adoption flow:
      1. validate payload
      2. add_wifi  → save wifi entry
      3. connect   → connect to network (rollback on failure)
      4. adopt_device → save config.json (rollback on failure)
    Returns {"ok": true} or {"ok": false, "error": "..."}.
    """
    if data is None:
        log_err(_MODULE, "handle_set_config", "missing payload")
        return {"ok": False, "error": "Missing payload"}

    log(_MODULE, "handle_set_config", "starting adoption | keys={}".format(list(data.keys())))

    if not validate_adoption_data(data):
        log_err(_MODULE, "handle_set_config", "validation failed")
        return {"ok": False, "error": "Invalid adoption payload"}

    ssid     = data.get("wifi_ssid", "")
    password = data.get("wifi_password", "")

    if not add_wifi(ssid, password):
        log_err(_MODULE, "handle_set_config", "failed to save wifi entry")
        return {"ok": False, "error": "Failed to save WiFi credentials"}

    log(_MODULE, "handle_set_config", "connecting to ssid='{}'".format(ssid))
    if not await connect(ssid, password):
        log_err(_MODULE, "handle_set_config", "wifi connection failed, rolling back")
        remove_wifi(ssid)
        return {"ok": False, "error": "Could not connect to WiFi network"}

    if not adopt_device(data):
        log_err(_MODULE, "handle_set_config", "adopt_device failed, rolling back wifi")
        remove_wifi(ssid)
        return {"ok": False, "error": "Failed to save device config"}

    global _adopted
    _adopted = True
    log(_MODULE, "handle_set_config", "adoption complete")
    return {"ok": True}


def handle_reboot(data):
    """Signal reboot — actual machine.reset() is called by serial_handler after response is sent."""
    log(_MODULE, "handle_reboot", "reboot requested")
    return {"ok": True}
