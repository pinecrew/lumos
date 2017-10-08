#!/usr/bin/env python
from time import sleep
from math import exp

max_backlight_file = "/sys/class/backlight/intel_backlight/max_brightness"
backlight_file = "/sys/class/backlight/intel_backlight/brightness"
illuminance_file = "/sys/bus/acpi/devices/ACPI0008:00/iio:device0/in_illuminance_raw"

min_backlight = 100
max_backlight = int(open(max_backlight_file).read())
min_illuminance = 0
max_illuminance = 7100

start, end = 0, 0
backlight = open(backlight_file, 'w+')
with open(illuminance_file, 'r') as f:
    while True:
        start = end
        end = int(f.read())
        # print('v =', end)
        f.seek(0)
        steps = abs(end - start) // 100 + 1
        print(steps)
        for i in range(steps):
            v = (start - end) * (1 / (exp((i-steps) / (0.5 * steps)) + 1)) + end
            print('v =', v)
            bv = int(min_backlight + (max_backlight - min_backlight) * ((v - min_illuminance) / (max_illuminance - min_illuminance))**0.5)
            # print('bv =', bv)
            backlight.seek(0)
            backlight.write(str(bv))
            sleep(0.125)
        sleep(0.1)