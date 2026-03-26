#!/bin/bash
echo "--- MEMORY & SWAP STATUS ---"
free -h
swapon --show

echo -e "\n--- ZSWAP STATUS ---"
grep -r . /sys/module/zswap/parameters/

echo -e "\n--- ZRAM STATUS ---"
zramctl

echo -e "\n--- DISK SWAP PRIORITY ---"
cat /proc/swaps

echo -e "\n--- SYSTEMD-BOOT CONFIG ---"
# List all boot entries to find the active one
ls /boot/loader/entries/
echo "Current Boot Parameters:"
cat /proc/cmdline
