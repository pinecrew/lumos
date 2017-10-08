#!/usr/bin/env python
from time import sleep
from math import exp

max_backlight_file = "/sys/class/backlight/intel_backlight/max_brightness"
backlight_file = "/sys/class/backlight/intel_backlight/brightness"
illuminance_file = "/sys/bus/acpi/devices/ACPI0008:00/iio:device0/in_illuminance_raw"

min_backlight = 100
max_backlight = int(open(max_backlight_file).read())
min_illuminance = 0
max_illuminance = 8036


def transition(x, center, range):
    return 1 / (exp(15 * (x-center) / range) + 1)


def lumos_to_backlight(v):
    x = (v - min_illuminance) / (max_illuminance - min_illuminance)
    return int(min_backlight + (max_backlight - min_backlight) * ((2 - x) * x)**0.5)


if __name__ == '__main__':
    start, end = 0, 0
    backlight = open(backlight_file, 'w+')
    with open(illuminance_file, 'r') as f:
        while True:
            start = end
            end = int(f.read())
            f.seek(0)
            steps = min(30, abs(end - start) // 10)
            print(steps, start, end)
            if steps:
                steps += 5
                for i in range(steps + 1):
                    v = (start - end) * transition(i, steps/2, steps) + end
                    print('v =', v)
                    bv = lumos_to_backlight(v)
                    print('bv =', bv)
                    backlight.seek(0)
                    backlight.write(str(bv))
                    sleep(0.05)
            else:
                backlight.seek(0)
                backlight.write(str(lumos_to_backlight(end)))
            sleep(0.1)