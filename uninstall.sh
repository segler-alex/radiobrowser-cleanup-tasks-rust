#!/bin/bash
sudo rm /usr/local/bin/update-caches.bin
sudo rm /usr/local/bin/update-caches
sudo userdel radiobrowser
sudo groupdel radiobrowser

sudo rm /etc/systemd/system/update-caches.service
sudo systemctl daemon-reload