import esp
import gc
import machine

esp.osdebug(None)       # silence vendor OS debug output
gc.collect()            # free memory before main.py runs
machine.freq(160000000) # 160MHz (ESP32-C3 max; full ESP32 supports 240MHz)
