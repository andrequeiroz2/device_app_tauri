"""
MQTT client module for embedded firmware.
Handles broker connection, topic subscription, data/status publishing,
and incoming command processing (RECONNECT, actuator commands).
"""

import json
import uasyncio as asyncio

from umqtt.simple import MQTTClient

from config.config  import read_config
from mqtt.mqtt_config import get_broker_params, update_broker_url
from tool.log       import log, log_err

_MODULE = "mqtt_client"

_client    = None
_topic     = None
_reconnect = False  # flag set by _handle_config to trigger reconnection

# MQTT parameters
_KEEPALIVE    = 60   # seconds — broker detects disconnect after 1.5x this value
_QOS_RELIABLE = 1    # at least once — commands, status changes
_QOS_SENSOR   = 0    # fire and forget — sensor data (time-sensitive, loss acceptable)


# --- Private helpers ---

def _on_message(raw_topic, raw_msg):
    """
    Callback invoked by umqtt on incoming messages.
    Routes by topic suffix: /config or /command.
    """
    try:
        topic = raw_topic.decode()
        msg   = raw_msg.decode()
    except Exception as e:
        log_err(_MODULE, "_on_message", "decode error", e)
        return

    log(_MODULE, "_on_message", "topic='{}' msg='{}'".format(topic, msg))

    if topic.endswith("/config"):
        _handle_config(msg)
    elif topic.endswith("/command"):
        _handle_command(msg)
    else:
        log(_MODULE, "_on_message", "unhandled topic suffix | topic='{}'".format(topic))


def _handle_config(msg):
    """
    Handle messages on {topic}/config.
    Supports action=RECONNECT: updates broker_url and sets reconnect flag.
    """
    global _reconnect
    try:
        payload = json.loads(msg)
    except ValueError as e:
        log_err(_MODULE, "_handle_config", "JSON parse error | msg='{}'".format(msg), e)
        return

    action = payload.get("action")
    log(_MODULE, "_handle_config", "action='{}'".format(action))

    if action == "RECONNECT":
        broker_url = payload.get("broker_url", "")
        if update_broker_url(broker_url):
            log(_MODULE, "_handle_config", "broker_url updated, scheduling reconnect")
            _reconnect = True
        else:
            log_err(_MODULE, "_handle_config", "failed to update broker_url='{}'".format(broker_url))
    else:
        log(_MODULE, "_handle_config", "unknown action='{}', ignoring".format(action))


def _handle_command(msg):
    """
    Handle messages on {topic}/command.
    Supports action=ON / action=OFF for actuators.
    """
    try:
        payload = json.loads(msg)
    except ValueError as e:
        log_err(_MODULE, "_handle_command", "JSON parse error | msg='{}'".format(msg), e)
        return

    action = payload.get("action")
    log(_MODULE, "_handle_command", "action='{}'".format(action))

    if action == "ON":
        log(_MODULE, "_handle_command", "actuator ON")
    elif action == "OFF":
        log(_MODULE, "_handle_command", "actuator OFF")
    else:
        log(_MODULE, "_handle_command", "unknown action='{}', ignoring".format(action))


def _build_client_id():
    """Build a unique client ID from config mac_address."""
    config = read_config()
    if config:
        mac = config.get("mac_address", "esp32")
        return "esp32_{}".format(mac.replace(":", ""))
    return "esp32_device"


# --- Public API ---

def is_connected():
    """Return True if MQTT client is connected, False otherwise."""
    return _client is not None


async def connect():
    """
    Read broker params from config.json, connect to broker,
    subscribe to {topic}/config and {topic}/command.
    Returns True on success, False on error.
    """
    global _client, _topic, _reconnect

    params = get_broker_params()
    if params is None:
        log_err(_MODULE, "connect", "could not get broker params")
        return False

    host, port, ssl = params

    config = read_config()
    if config is None:
        log_err(_MODULE, "connect", "could not read config.json")
        return False

    _topic     = config.get("topic", "")
    client_id  = _build_client_id()
    _reconnect = False

    log(_MODULE, "connect", "connecting to host='{}' port={} ssl={} topic='{}'".format(
        host, port, ssl, _topic))

    will_topic = "{}/status".format(_topic).encode()
    will_msg   = '{"state":"offline"}'.encode()

    try:
        _client = MQTTClient(
            client_id,
            host,
            port=port,
            ssl=ssl,
            keepalive=_KEEPALIVE,
        )
        _client.set_callback(_on_message)
        _client.set_last_will(will_topic, will_msg, retain=True, qos=_QOS_RELIABLE)
        _client.connect()
        log(_MODULE, "connect", "connected client_id='{}' keepalive={}s".format(client_id, _KEEPALIVE))
    except Exception as e:
        log_err(_MODULE, "connect", "connection failed", e)
        _client = None
        return False

    try:
        _client.subscribe("{}/config".format(_topic).encode(),   _QOS_RELIABLE)
        _client.subscribe("{}/command".format(_topic).encode(),  _QOS_RELIABLE)
        log(_MODULE, "connect", "subscribed to {}/config and {}/command".format(_topic, _topic))
    except Exception as e:
        log_err(_MODULE, "connect", "subscribe failed", e)
        return False

    publish_status("online")
    return True


def disconnect():
    """Disconnect from broker and release client."""
    global _client
    if _client is None:
        return
    try:
        publish_status("offline")
        _client.disconnect()
        log(_MODULE, "disconnect", "disconnected")
    except Exception as e:
        log_err(_MODULE, "disconnect", "error during disconnect", e)
    finally:
        _client = None


def publish_data(payload):
    """
    Publish sensor data to {topic}/data.
    payload: dict e.g. {"temperature": 25, "humidity": 60}
    """
    if _client is None:
        log_err(_MODULE, "publish_data", "client not connected")
        return

    try:
        msg = json.dumps(payload)
        _client.publish("{}/data".format(_topic).encode(), msg.encode(), qos=_QOS_SENSOR)
        log(_MODULE, "publish_data", "published: {}".format(msg))
    except Exception as e:
        log_err(_MODULE, "publish_data", "publish failed", e)


def publish_status(state):
    """
    Publish heartbeat/state to {topic}/status.
    state: "online" or "offline"
    """
    if _client is None:
        log_err(_MODULE, "publish_status", "client not connected")
        return

    try:
        msg = json.dumps({"state": state})
        _client.publish("{}/status".format(_topic).encode(), msg.encode(), qos=_QOS_RELIABLE, retain=True)
        log(_MODULE, "publish_status", "state='{}'".format(state))
    except Exception as e:
        log_err(_MODULE, "publish_status", "publish failed", e)


def poll():
    """
    Check for pending MQTT messages without blocking.
    Must be called on every main loop cycle.
    Returns True if a reconnect was requested (caller should await connect()).
    """
    if _client is None:
        return False

    try:
        _client.check_msg()
    except Exception as e:
        log_err(_MODULE, "poll", "check_msg error", e)

    return _reconnect
