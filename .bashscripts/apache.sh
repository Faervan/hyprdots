#!/bin/bash

if [ $1 = 'start' ]; then
	echo "starting apache webserver..."
	sudo systemctl enable httpd
	sudo systemctl start httpd
	systemctl status httpd
	echo "Your IP address (inet) is: " && ip -o address | grep "wlp4s0    inet "
elif [ $1 = 'stop' ]; then
	echo "stopping apache webserver..."
	sudo systemctl stop httpd
	sudo systemctl disable httpd
	systemctl status httpd | grep -E "httpd.service - Apache Web Server|Loaded|Active"
else
	echo "unsupported option"
fi
