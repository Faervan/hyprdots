#!/bin/bash
brightness=$(cat /sys/class/backlight/intel_backlight/brightness)
keynum='76'
if [ $1 == '+5%' ]; then
	((brightness += keynum))
elif [ $1 == '-5%' ]; then
	((brightness -= keynum))
else
	echo "invalid option"
fi
if [ $1 == '+5%' ] && [ $brightness -ge '1515' ]; then
	echo "max"
	((brightness = 1515))
elif [ $1 == '-5%' ] && [ $brightness -le '0' ]; then
	echo "min"
	((brightness = 0))
fi
echo $brightness > /sys/class/backlight/intel_backlight/brightness
