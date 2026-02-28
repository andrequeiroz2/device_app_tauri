"""
Configuration module for device embedded firmware.
Manages persistent config.json (load, read, write, update).
"""

import json
import network
import os

# --- Constants ---

CONFIG_FILE = "/config.json"

# Default values
BOARDER_TYPE = "ESP32"
DEVICE_TYPE = "Sensor"
SENSOR_TYPE = "DHT11"
ACTUATOR_TYPE = ""
DEVICE_SCALE = [["temperature", "C"], ["humidity", "%"]]
ADOPTED_STATUS = 0
ADOPTED_STATUS_DESC = "not_adopted"


class AdoptedStatus:
    """Adoption status constants."""

    UNADOPTED = 0
    ADOPTED = 1
    DESC = {0: "not_adopted", 1: "adopted"}


async def load_config() -> bool:
    """
    Check if config file exists.
    Returns True if file exists, False otherwise (e.g. OSError).
    """
    try:
        os.stat(CONFIG_FILE)
        return True
    except OSError as e:
        print("config: load_config error:", e)
        return False


async def read_config() -> dict | None:
    """
    Read and parse config.json.
    Returns config dict or None if file is invalid/missing.
    """
    try:
        with open(CONFIG_FILE, "r", encoding="utf-8") as f:
            return json.load(f)
    except (OSError, ValueError):
        return None


async def get_mac_address() -> str | None:
    """
    Get MAC address from WLAN interface.
    Returns formatted string (e.g. "3C:71:BF:4D:DB:0C") or None on error.
    """
    try:
        wlan = network.WLAN(network.STA_IF)
        if not wlan.active():
            wlan.active(True)
        mac = wlan.config("mac")
        return ":".join("{:02X}".format(b) for b in mac)
    except (OSError, AttributeError):
        return None


def _is_config_complete(config: dict) -> bool:
    """Check if config has all required device-side fields."""
    required = (
        "boarder_type",
        "mac_address",
        "device_type",
        "sensor_type",
        "actuator_type",
        "adopted_status",
        "adopted_status_desc",
        "device_scale",
    )
    for key in required:
        if key not in config:
            return False
    if not config.get("mac_address"):
        return False
    if not isinstance(config.get("device_scale"), list):
        return False
    return True


def _build_init_config(mac: str) -> dict:
    """Build initial config dict with defaults and MAC."""
    return {
        "adopted_status": AdoptedStatus.UNADOPTED,
        "adopted_status_desc": AdoptedStatus.DESC[AdoptedStatus.UNADOPTED],
        "device_type": DEVICE_TYPE,
        "sensor_type": SENSOR_TYPE,
        "actuator_type": ACTUATOR_TYPE,
        "boarder_type": BOARDER_TYPE,
        "mac_address": mac,
        "device_scale": DEVICE_SCALE,
        "broker_url": "",
        "topic": "",
        "user_uuid": "",
        "device_uuid": "",
        "device_name": "",
        "wifi_ssid": "",
        "wifi_password": "",
    }


async def write_init_config() -> bool:
    """
    Ensure config.json exists with valid initial content.
    Creates file if missing, fills missing fields if incomplete.
    Returns True on success, False if MAC unavailable.
    """
    if not await load_config():
        mac = await get_mac_address()
        if mac is None:
            return False
        config = _build_init_config(mac)
        try:
            with open(CONFIG_FILE, "w", encoding="utf-8") as f:
                json.dump(config, f)
        except OSError:
            return False
        return True

    config = await read_config()
    if config is None:
        mac = await get_mac_address()
        if mac is None:
            return False
        config = _build_init_config(mac)
        try:
            with open(CONFIG_FILE, "w", encoding="utf-8") as f:
                json.dump(config, f)
        except OSError:
            return False
        return True

    if _is_config_complete(config):
        return True

    mac = await get_mac_address()
    if mac is None:
        return False
    config = _build_init_config(mac)
    try:
        with open(CONFIG_FILE, "w", encoding="utf-8") as f:
            json.dump(config, f)
    except OSError:
        return False
    return True


_REQUIRED_UPDATE_KEYS = (
    "user_uuid",
    "device_uuid",
    "device_name",
    "topic",
    "broker_url",
    "wifi_ssid",
)
_UPDATE_KEYS = (
    "device_name",
    "user_uuid",
    "topic",
    "broker_url",
    "wifi_ssid",
    "wifi_password",
    "adopted_status",
    "adopted_status_desc",
)


def _validate_update_data(data: dict) -> bool:
    """Validate set_config payload: required fields must be non-empty strings."""
    for key in _REQUIRED_UPDATE_KEYS:
        val = data.get(key)
        if not isinstance(val, str) or not val.strip():
            return False
    # wifi_password can be empty (open network)
    if "wifi_password" in data and not isinstance(data.get("wifi_password"), str):
        return False
    # adopted_status must be 0 or 1
    status = data.get("adopted_status")
    if status not in (0, 1):
        return False
    # adopted_status_desc must be string
    if "adopted_status_desc" in data and not isinstance(
        data.get("adopted_status_desc"), str
    ):
        return False
    return True


async def update_config(data: dict) -> bool:
    """
    Update config with set_config payload (adoption data).
    Returns True on success, False if validation fails or already adopted.
    """
    if not _validate_update_data(data):
        return False

    config = await read_config()
    if config is None:
        return False

    # Reject only if a different user tries to adopt an already-adopted device
    if (
        config.get("adopted_status") == AdoptedStatus.ADOPTED
        and data.get("user_uuid") != config.get("user_uuid")
    ):
        return False

    for key in _UPDATE_KEYS:
        if key in data:
            config[key] = data[key]
    # device_uuid: set only on first adoption, never overwrite
    if "device_uuid" in data and not config.get("device_uuid"):
        config["device_uuid"] = data["device_uuid"]

    try:
        with open(CONFIG_FILE, "w", encoding="utf-8") as f:
            json.dump(config, f)
    except OSError:
        return False
    return True
