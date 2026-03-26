#!/bin/bash

# 1. Remove the malformed modprobe file
if [ -f /etc/modprobe.d/zswap.conf ]; then
    echo "Removing broken zswap.conf..."
    sudo rm /etc/modprobe.d/zswap.conf
fi

# 2. Update systemd-boot entry
# We look for the entry matching your current initrd (initramfs-linux-g14.img)
ENTRY_FILE=$(sudo grep -l "initramfs-linux-g14.img" /boot/loader/entries/*.conf)

if [ -n "$ENTRY_FILE" ]; then
    echo "Found active boot entry: $ENTRY_FILE"
    # Check if zswap.enabled=0 is already there to avoid duplicates
    if ! grep -q "zswap.enabled=0" "$ENTRY_FILE"; then
        echo "Appending zswap.enabled=0 to $ENTRY_FILE..."
        sudo sed -i '/^options/ s/$/ zswap.enabled=0/' "$ENTRY_FILE"
    else
        echo "zswap.enabled=0 already present in boot entry."
    fi
else
    echo "Error: Could not find the active systemd-boot entry file."
fi

# 3. Rebuild initcpio to verify the error is gone
echo "Rebuilding initcpio..."
sudo mkinitcpio -P

echo "Done! Please reboot for changes to take effect."
