#!/bin/bash
backlight=$(cat /sys/class/leds/tpacpi::kbd_backlight/brightness)
if [ $1 == '-1' ]; then
	((backlight -= 1))
elif [ $1 == '+1' ]; then
	((backlight += 1))
else
	echo "invalid option"
fi
if [ $backlight -ge 2 ]; then
	echo "max"
	((backlight = 2))
elif [ $backlight -le 0 ]; then
	echo "min"
	((backlight = 0))
fi
echo $backlight > /sys/class/leds/tpacpi::kbd_backlight/brightness

