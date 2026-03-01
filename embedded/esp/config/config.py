"""
Configuration module for device embedded firmware.
Manages persistent config.json (load, read, write, update).
"""

from tool.load import file_exists
from tool.log import log, log_err
from tool.network import get_mac_address
from tool.read import read_json
from tool.validate import validate_adoption_data, ADOPTION_KEYS
from tool.write import write_json

_MODULE = "config"

# --- Constants ---

CONFIG_FILE = "/config.json"

BOARDER_TYPE        = "ESP32"
DEVICE_TYPE         = "Sensor"
SENSOR_TYPE         = "DHT11"
ACTUATOR_TYPE       = ""
DEVICE_SCALE        = [["temperature", "C"], ["humidity", "%"]]
ADOPTED_STATUS      = 0
ADOPTED_STATUS_DESC = "not_adopted"


class AdoptedStatus:
    """Adoption status constants."""
    UNADOPTED = 0
    ADOPTED   = 1
    DESC      = {0: "not_adopted", 1: "adopted"}


# --- Private helpers ---

def _is_config_complete(config):
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


def _build_init_config(mac):
    """Build initial config dict with defaults and MAC."""
    return {
        "adopted_status":      AdoptedStatus.UNADOPTED,
        "adopted_status_desc": AdoptedStatus.DESC[AdoptedStatus.UNADOPTED],
        "device_type":         DEVICE_TYPE,
        "sensor_type":         SENSOR_TYPE,
        "actuator_type":       ACTUATOR_TYPE,
        "boarder_type":        BOARDER_TYPE,
        "mac_address":         mac,
        "device_scale":        DEVICE_SCALE,
        "broker_url":          "",
        "topic":               "",
        "user_uuid":           "",
        "device_uuid":         "",
        "device_name":         "",
    }


def read_config():
    """Read and parse config.json. Returns dict or None."""
    return read_json(CONFIG_FILE)


def write_init_config():
    """
    Ensure config.json exists with valid initial content.
    Creates file if missing or incomplete.
    Returns True on success, False if MAC unavailable or write error.
    """
    if not file_exists(CONFIG_FILE):
        log(_MODULE, "write_init_config", "config.json not found, creating")
        mac = get_mac_address()
        if mac is None:
            log_err(_MODULE, "write_init_config", "MAC address unavailable, cannot create config")
            return False
        return write_json(CONFIG_FILE, _build_init_config(mac))

    config = read_config()
    if config is None:
        log_err(_MODULE, "write_init_config", "config.json exists but is unreadable, recreating")
        mac = get_mac_address()
        if mac is None:
            log_err(_MODULE, "write_init_config", "MAC address unavailable, cannot recreate config")
            return False
        return write_json(CONFIG_FILE, _build_init_config(mac))

    if _is_config_complete(config):
        return True

    log(_MODULE, "write_init_config", "config.json is incomplete, rebuilding")
    mac = get_mac_address()
    if mac is None:
        log_err(_MODULE, "write_init_config", "MAC address unavailable, cannot rebuild config")
        return False
    return write_json(CONFIG_FILE, _build_init_config(mac))


def adopt_device(data):
    """
    Adopt the device by applying the set_config payload.
    Validates all connectivity fields required for operation mode.
    Returns True on success, False if validation fails or device is already
    adopted by a different user.
    """
    log(_MODULE, "adopt_device", "received keys={}".format(list(data.keys())))

    if not validate_adoption_data(data):
        log_err(_MODULE, "adopt_device", "adoption payload validation failed | data={}".format(data))
        return False

    config = read_config()
    if config is None:
        log_err(_MODULE, "adopt_device", "could not read config.json")
        return False

    current_status = config.get("adopted_status")
    current_user   = config.get("user_uuid")
    incoming_user  = data.get("user_uuid")

    if current_status == AdoptedStatus.ADOPTED and incoming_user != current_user:
        log_err(_MODULE, "adopt_device", "already adopted by another user | current_user={}".format(current_user))
        return False

    for key in ADOPTION_KEYS:
        if key in data:
            log(_MODULE, "adopt_device", "set: {}='{}' -> '{}'".format(key, config.get(key, ""), data[key]))
            config[key] = data[key]

    if "device_uuid" in data and not config.get("device_uuid"):
        log(_MODULE, "adopt_device", "device_uuid set (first adoption): {}".format(data["device_uuid"]))
        config["device_uuid"] = data["device_uuid"]

    return write_json(CONFIG_FILE, config)
