#!/bin/bash

# This script configures Arch Linux to use NetworkManager with the iwd backend
# for Wi-Fi management. It resolves conflicts by disabling other network
# services like wpa_supplicant and systemd-networkd.

# Ensure the script is run with root privileges
if [ "$EUID" -ne 0 ]; then
  echo "This script must be run as root. Please use sudo."
  exit 1
fi

echo "--- Stopping all potentially conflicting network services ---"
systemctl stop NetworkManager iwd wpa_supplicant systemd-networkd

echo "--- Disabling conflicting services from starting on boot ---"
systemctl disable wpa_supplicant systemd-networkd
systemctl disable wpa_supplicant.service # Also disable the specific service file if it exists
systemctl disable systemd-networkd.service

echo "--- Configuring NetworkManager to use iwd as the Wi-Fi backend ---"
# Create the configuration directory if it doesn't exist
mkdir -p /etc/NetworkManager/conf.d

# Write the configuration file
tee /etc/NetworkManager/conf.d/wifi_backend.conf > /dev/null <<EOF
[device]
wifi.backend=iwd
EOF

echo "--- Enabling the required services: NetworkManager and iwd ---"
systemctl enable iwd.service
systemctl enable NetworkManager.service

echo "--- Starting NetworkManager (which will manage iwd) ---"
systemctl start NetworkManager

echo ""
echo "--- Configuration Complete ---"
echo "NetworkManager and iwd have been configured and started."
echo "Your system should now have a stable Wi-Fi connection."
echo "You may need to reconnect to your Wi-Fi network using NetworkManager."