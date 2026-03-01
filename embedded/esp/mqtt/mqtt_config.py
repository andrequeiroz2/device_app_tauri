"""
MQTT configuration module.
Handles broker URL updates triggered by remote RECONNECT commands.
"""

from config.config import AdoptedStatus
from tool.log import log, log_err
from tool.read import read_json
from tool.write import write_json

CONFIG_FILE = "/config.json"
_MODULE     = "mqtt_config"


# --- Private helpers ---

def _parse_broker_url(broker_url):
    """
    Parse broker_url into (host, port, ssl).

    Supported formats:
      mqtt://host:port   -> ssl=False
      mqtts://host:port  -> ssl=True
      mqtt://host        -> port=1883, ssl=False
      mqtts://host       -> port=8883, ssl=True

    Returns (host, port, ssl) or None if invalid.
    """
    log(_MODULE, "_parse_broker_url", "input='{}'".format(broker_url))

    if not isinstance(broker_url, str) or not broker_url.strip():
        log_err(_MODULE, "_parse_broker_url", "invalid type or empty | type={}".format(type(broker_url)))
        return None

    if broker_url.startswith("mqtts://"):
        ssl          = True
        address      = broker_url[8:]
        default_port = 8883
        log(_MODULE, "_parse_broker_url", "scheme=mqtts address='{}'".format(address))
    elif broker_url.startswith("mqtt://"):
        ssl          = False
        address      = broker_url[7:]
        default_port = 1883
        log(_MODULE, "_parse_broker_url", "scheme=mqtt address='{}'".format(address))
    else:
        log_err(_MODULE, "_parse_broker_url", "unsupported scheme | value='{}'".format(broker_url))
        return None

    if not address:
        log_err(_MODULE, "_parse_broker_url", "empty address after scheme")
        return None

    parts = address.split(":")
    host  = parts[0]

    if not host:
        log_err(_MODULE, "_parse_broker_url", "empty host | address='{}'".format(address))
        return None

    if len(parts) == 2:
        try:
            port = int(parts[1])
            log(_MODULE, "_parse_broker_url", "explicit port={}".format(port))
        except ValueError as e:
            log_err(_MODULE, "_parse_broker_url", "invalid port | raw='{}'".format(parts[1]), e)
            return None
    else:
        port = default_port
        log(_MODULE, "_parse_broker_url", "default port={}".format(port))

    log(_MODULE, "_parse_broker_url", "result host='{}' port={} ssl={}".format(host, port, ssl))
    return (host, port, ssl)


# --- Public API ---

def update_broker_url(broker_url):
    """
    Update broker_url in config.json.
    Called when device receives {"action":"RECONNECT","broker_url":"..."} via MQTT.
    Returns True on success, False on validation or write error.
    """
    log(_MODULE, "update_broker_url", "received broker_url='{}'".format(broker_url))

    parsed = _parse_broker_url(broker_url)
    if parsed is None:
        log_err(_MODULE, "update_broker_url", "invalid broker_url, aborting")
        return False

    host, port, ssl = parsed
    log(_MODULE, "update_broker_url", "validated host='{}' port={} ssl={}".format(host, port, ssl))

    config = read_json(CONFIG_FILE)
    if config is None:
        log_err(_MODULE, "update_broker_url", "could not read config.json")
        return False

    adopted = config.get("adopted_status")
    if adopted != AdoptedStatus.ADOPTED:
        log_err(_MODULE, "update_broker_url", "device not adopted | adopted_status={}".format(adopted))
        return False

    previous = config.get("broker_url", "")
    config["broker_url"] = broker_url
    log(_MODULE, "update_broker_url", "broker_url: '{}' -> '{}'".format(previous, broker_url))

    return write_json(CONFIG_FILE, config)


def get_broker_params():
    """
    Read broker connection parameters from config.json.
    Returns (host, port, ssl) or None if unavailable or unparseable.
    """
    log(_MODULE, "get_broker_params", "reading from config.json")

    config = read_json(CONFIG_FILE)
    if config is None:
        log_err(_MODULE, "get_broker_params", "could not read config.json")
        return None

    broker_url = config.get("broker_url", "")
    log(_MODULE, "get_broker_params", "broker_url='{}'".format(broker_url))

    if not broker_url:
        log_err(_MODULE, "get_broker_params", "broker_url is empty")
        return None

    result = _parse_broker_url(broker_url)
    if result is None:
        log_err(_MODULE, "get_broker_params", "failed to parse broker_url='{}'".format(broker_url))
        return None

    host, port, ssl = result
    log(_MODULE, "get_broker_params", "returning host='{}' port={} ssl={}".format(host, port, ssl))
    return result
