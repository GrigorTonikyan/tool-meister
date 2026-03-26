#!/bin/bash
set -e

WINDOWS_ESP_DEV="/dev/nvme0n1p1"
MOUNT_POINT="/mnt/windows-efi"
LINUX_ESP="/boot"

# Check if Windows ESP exists
if [ ! -b "$WINDOWS_ESP_DEV" ]; then
    echo "Error: Windows ESP device $WINDOWS_ESP_DEV not found."
    exit 1
fi

echo "Mounting Windows ESP ($WINDOWS_ESP_DEV) to $MOUNT_POINT..."
mkdir -p "$MOUNT_POINT"
mount "$WINDOWS_ESP_DEV" "$MOUNT_POINT"

if [ ! -d "$MOUNT_POINT/EFI/Microsoft" ]; then
    echo "Error: EFI/Microsoft directory not found on Windows ESP."
    umount "$MOUNT_POINT"
    exit 1
fi

echo "Found Windows Boot Manager."

echo "Copying EFI/Microsoft to $LINUX_ESP/EFI/..."
mkdir -p "$LINUX_ESP/EFI"

# Use rsync to copy/update, preserving permissions/times
# or just cp -r
cp -r "$MOUNT_POINT/EFI/Microsoft" "$LINUX_ESP/EFI/"

echo "Unmounting Windows ESP..."
umount "$MOUNT_POINT"
rmdir "$MOUNT_POINT" || true

echo "Done. Checking bootctl list..."
bootctl list | grep "Microsoft" || echo "Warning: Windows entry might not have appeared yet or output format differs."

echo "Windows EFI files copied. systemd-boot should auto-detect 'EFI/Microsoft/Boot/bootmgfw.efi'."
