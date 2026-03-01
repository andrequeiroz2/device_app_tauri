"""
Validation utility for embedded firmware.
Provides reusable payload validators.
"""

from tool.log import log, log_err

_MODULE = "validate"

REQUIRED_ADOPTION_KEYS = (
    "user_uuid",
    "device_uuid",
    "topic",
    "broker_url",
    "wifi_ssid",
)

ADOPTION_KEYS = (
    "device_name",
    "user_uuid",
    "topic",
    "broker_url",
    "adopted_status",
    "adopted_status_desc",
)


def validate_adoption_data(data):
    """
    Validate adoption payload (set_config).
    Required fields must be non-empty strings.
    wifi_ssid is required and validated here but routed to wifi.json, not config.json.
    wifi_password is optional but must be str if present.
    adopted_status must be 0 or 1.
    Returns True if valid, False otherwise.
    """
    log(_MODULE, "validate_adoption_data", "validating keys={}".format(list(data.keys())))

    for key in REQUIRED_ADOPTION_KEYS:
        val = data.get(key)
        if not isinstance(val, str) or not val.strip():
            log_err(_MODULE, "validate_adoption_data", "missing or empty required field | key='{}'".format(key))
            return False

    if "wifi_password" in data and not isinstance(data.get("wifi_password"), str):
        log_err(_MODULE, "validate_adoption_data", "wifi_password must be str | type={}".format(type(data.get("wifi_password"))))
        return False

    status = data.get("adopted_status")
    if status not in (0, 1):
        log_err(_MODULE, "validate_adoption_data", "invalid adopted_status | value={}".format(status))
        return False

    if "adopted_status_desc" in data and not isinstance(data.get("adopted_status_desc"), str):
        log_err(_MODULE, "validate_adoption_data", "adopted_status_desc must be str")
        return False

    log(_MODULE, "validate_adoption_data", "payload is valid")
    return True
