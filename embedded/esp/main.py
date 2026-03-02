"""
Main entry point for embedded firmware.
Orchestrates boot sequence and runs the main loop in one of two modes:
  - Configuration mode (adopted_status == 0): listen for serial commands only
  - Operation mode    (adopted_status == 1): WiFi + MQTT + sensor readings
"""

import gc
import uasyncio as asyncio
from machine import Timer, WDT

from config.config           import write_init_config, read_config, AdoptedStatus
from mqtt.mqtt_client        import connect as mqtt_connect, disconnect as mqtt_disconnect, poll as mqtt_poll, publish_data, publish_status
from protocol.commands       import is_adopted
from protocol.serial_handler import init as serial_init, poll as serial_poll
from sensors.dht             import init as dht_init, read as dht_read
from tool.log                import log, log_err
from wifi.config_wifi        import write_init_config as wifi_init, connect_from_config

_MODULE      = "main"
DHT_PIN      = 4     # GPIO pin connected to DHT11 data line
HEARTBEAT_S  = 30    # publish_status("online") interval in seconds
SENSOR_S     = 10    # sensor read + publish interval in seconds
WDT_TIMEOUT  = 8000  # watchdog timeout in milliseconds

# Timer flags — set inside Timer callbacks, consumed in the main loop
_sensor_flag    = False
_heartbeat_flag = False


def _on_sensor_tick(t):
    global _sensor_flag
    _sensor_flag = True


def _on_heartbeat_tick(t):
    global _heartbeat_flag
    _heartbeat_flag = True


async def _run_config_mode(wdt):
    """
    Configuration mode loop.
    Device is not adopted — only listens for serial commands (ping, get_info, set_config, reboot).
    Exits when adopted_status transitions to ADOPTED after a successful set_config.
    """
    log(_MODULE, "_run_config_mode", "entering configuration mode")

    while True:
        await serial_poll()

        if is_adopted():
            log(_MODULE, "_run_config_mode", "device adopted, switching to operation mode")
            return

        wdt.feed()
        await asyncio.sleep_ms(50)


async def _run_operation_mode(wdt):
    """
    Operation mode loop.
    Connects WiFi and MQTT broker, publishes sensor data and heartbeat,
    handles incoming MQTT commands and serial commands.
    """
    global _sensor_flag, _heartbeat_flag

    log(_MODULE, "_run_operation_mode", "entering operation mode")

    # Connect WiFi
    log(_MODULE, "_run_operation_mode", "connecting WiFi")
    if not await connect_from_config():
        log_err(_MODULE, "_run_operation_mode", "WiFi connection failed — staying in operation mode loop")

    # Connect MQTT broker
    log(_MODULE, "_run_operation_mode", "connecting MQTT broker")
    if not await mqtt_connect():
        log_err(_MODULE, "_run_operation_mode", "MQTT connection failed — will retry on reconnect flag")

    # Start sensor and heartbeat timers
    dht_init(DHT_PIN)

    sensor_timer    = Timer(0)
    heartbeat_timer = Timer(1)

    sensor_timer.init(
        period=SENSOR_S * 1000,
        mode=Timer.PERIODIC,
        callback=_on_sensor_tick,
    )
    heartbeat_timer.init(
        period=HEARTBEAT_S * 1000,
        mode=Timer.PERIODIC,
        callback=_on_heartbeat_tick,
    )

    log(_MODULE, "_run_operation_mode", "timers started — sensor={}s heartbeat={}s".format(
        SENSOR_S, HEARTBEAT_S))

    try:
        while True:
            # Serial commands (reboot, get_info still available in operation mode)
            await serial_poll()

            # MQTT incoming messages — returns True if RECONNECT was received
            needs_reconnect = mqtt_poll()
            if needs_reconnect:
                log(_MODULE, "_run_operation_mode", "RECONNECT requested, reconnecting broker")
                mqtt_disconnect()
                await mqtt_connect()

            # Sensor read tick
            if _sensor_flag:
                _sensor_flag = False
                data = dht_read()
                if data is not None:
                    publish_data(data)
                else:
                    log_err(_MODULE, "_run_operation_mode", "sensor read returned None")

            # Heartbeat tick
            if _heartbeat_flag:
                _heartbeat_flag = False
                publish_status("online")

            gc.collect()
            wdt.feed()
            await asyncio.sleep_ms(50)

    finally:
        sensor_timer.deinit()
        heartbeat_timer.deinit()
        mqtt_disconnect()


async def main():
    # --- Boot sequence ---
    log(_MODULE, "main", "boot sequence started")

    write_init_config()
    wifi_init()
    serial_init()

    wdt = WDT(timeout=WDT_TIMEOUT)
    log(_MODULE, "main", "watchdog started timeout={}ms".format(WDT_TIMEOUT))

    # --- Mode selection ---
    config = read_config()
    if config and config.get("adopted_status") == AdoptedStatus.ADOPTED:
        await _run_operation_mode(wdt)
    else:
        await _run_config_mode(wdt)
        # After adoption completes, transition to operation mode
        await _run_operation_mode(wdt)


asyncio.run(main())
