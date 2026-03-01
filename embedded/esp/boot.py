import esp
import gc
import machine

esp.osdebug(None)       # silence vendor OS debug output
gc.collect()            # free memory before main.py runs
machine.freq(240000000) # set CPU to 240MHz for better serial/WiFi performance
