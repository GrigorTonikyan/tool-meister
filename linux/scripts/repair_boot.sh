#!/bin/bash
set -e

# Check for sudo
if [ "$EUID" -ne 0 ]; then 
  echo "Please run as root (sudo)"
  exit 1
fi

echo "Retrieving Root UUID from /etc/fstab..."
# Extract UUID for / mount point
ROOT_UUID=$(grep "[[:space:]]/[[:space:]]" /etc/fstab | grep -o "UUID=[^[:space:]]*" | cut -d= -f2)

if [ -z "$ROOT_UUID" ]; then
    echo "Could not find Root UUID in fstab. Trying alternative match..."
    ROOT_UUID=$(grep "[[:space:]]/[[:space:]]" /etc/fstab | awk '{print $1}' | sed 's/UUID=//')
fi

if [ -z "$ROOT_UUID" ] || [[ "$ROOT_UUID" == "/"* ]]; then
    echo "Error: Could not determine Root UUID. Found: $ROOT_UUID"
    echo "Please check /etc/fstab manually."
    exit 1
fi

echo "Detected Root UUID: $ROOT_UUID"

# Function to update or create boot entry
update_entry() {
    local name=$1
    local title=$2
    local vmlinuz=$3
    local initrd=$4
    
    local entry_file="/boot/loader/entries/${name}.conf"
    
    echo "Updating $entry_file..."
    
    # Basic options
    local boot_options="root=UUID=$ROOT_UUID rw quiet splash"
    
    # Check if file exists and has options line, try to preserve it?
    # For now, we overwrite because user said it's broken/stale.
    
    cat <<EOF > "$entry_file"
title   $title
linux   $vmlinuz
initrd  $initrd
options $boot_options
EOF
}

# 1. Handle Linux G14
echo "------------------------------------------------"
echo "Processing Linux G14..."
G14_PATH=$(find /usr/lib/modules -maxdepth 1 -name "*-g14" | sort -V | tail -n 1)
if [ -n "$G14_PATH" ] && [ -f "$G14_PATH/vmlinuz" ]; then
    echo "Found kver: $(basename "$G14_PATH")"
    cp -v "$G14_PATH/vmlinuz" /boot/vmlinuz-linux-g14
    update_entry "linux-g14" "Arch Linux (G14)" "/vmlinuz-linux-g14" "/initramfs-linux-g14.img"
else
    echo "Warning: No G14 kernel found in /usr/lib/modules!"
fi

# 2. Handle Standard Linux
echo "------------------------------------------------"
echo "Processing Standard Linux..."
STD_PATH=$(find /usr/lib/modules -maxdepth 1 -name "*arch*" -not -name "*-g14" | sort -V | tail -n 1)
if [ -n "$STD_PATH" ] && [ -f "$STD_PATH/vmlinuz" ]; then
    echo "Found kver: $(basename "$STD_PATH")"
    cp -v "$STD_PATH/vmlinuz" /boot/vmlinuz-linux
    update_entry "arch" "Arch Linux" "/vmlinuz-linux" "/initramfs-linux.img"
    # Fallback
    echo "Updating Fallback entry..."
    # Reuse options but change initrd
    cat <<EOF > "/boot/loader/entries/arch-fallback.conf"
title   Arch Linux (Fallback)
linux   /vmlinuz-linux
initrd  /initramfs-linux-fallback.img
options root=UUID=$ROOT_UUID rw quiet splash
EOF
else
    echo "Warning: No Standard kernel found!"
fi

# 3. Regenerate Initramfs
echo "------------------------------------------------"
echo "Regenerating initramfs images (mkinitcpio)..."
mkinitcpio -P

# 4. Create Pacman Hook
echo "------------------------------------------------"
echo "Installing Pacman hook for kernel updates..."

HOOK_SCRIPT="/usr/local/bin/copy-kernels.sh"
cat <<'EOF' > "$HOOK_SCRIPT"
#!/bin/bash
# Copy G14
for k in /usr/lib/modules/*-g14; do
    if [ -f "$k/vmlinuz" ]; then
        cp -f "$k/vmlinuz" /boot/vmlinuz-linux-g14
    fi
done
# Copy Standard
for k in /usr/lib/modules/*-arch*; do
    if [[ "$k" != *"-g14" ]] && [ -f "$k/vmlinuz" ]; then
         cp -f "$k/vmlinuz" /boot/vmlinuz-linux
    fi
done
EOF
chmod +x "$HOOK_SCRIPT"

mkdir -p /etc/pacman.d/hooks
cat <<EOF > /etc/pacman.d/hooks/99-copy-kernels.hook
[Trigger]
Type = Path
Operation = Install
Operation = Upgrade
Target = usr/lib/modules/*/vmlinuz

[Action]
Description = Copying kernels to /boot
When = PostTransaction
Exec = $HOOK_SCRIPT
EOF

echo "------------------------------------------------"
echo "Fix complete. Please check /boot/loader/entries/ to ensure options are correct."
echo "You may need to add 'nvidia_drm.modeset=1' to options if using Nvidia graphics."
