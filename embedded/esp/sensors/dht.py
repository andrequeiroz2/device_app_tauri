"""
DHT sensor module for embedded firmware.
Wrapper around MicroPython's built-in dht driver with logging and error handling.
Supports DHT11 (integer readings) and DHT22 (float readings).
Read interval: minimum 10 seconds (enforced by the caller via Timer).
"""

import dht
from machine import Pin

from tool.log import log, log_err

_MODULE = "dht"
_sensor = None


def init(pin):
    """
    Initialize DHT11 sensor on the given GPIO pin number.
    Must be called once before read().
    """
    global _sensor
    _sensor = dht.DHT11(Pin(pin))
    log(_MODULE, "init", "DHT11 initialized on pin={}".format(pin))


def read():
    """
    Trigger a measurement and return sensor data.
    measure() is blocking (~15ms for DHT11).
    Returns {"temperature": t, "humidity": h} or None on error.
    """
    if _sensor is None:
        log_err(_MODULE, "read", "sensor not initialized — call init(pin) first")
        return None

    try:
        _sensor.measure()
        temperature = _sensor.temperature()
        humidity    = _sensor.humidity()
        log(_MODULE, "read", "temperature={} humidity={}".format(temperature, humidity))
        return {"temperature": temperature, "humidity": humidity}
    except Exception as e:
        log_err(_MODULE, "read", "measurement failed", e)
        return None
