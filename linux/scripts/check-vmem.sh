#!/bin/bash

echo "--- 1. GPU IDENTIFICATION ---"
lspci -nnk | grep -A 3 "VGA"

echo -e "\n--- 2. i915 KERNEL PARAMETERS ---"
# Check what parameters are currently active in the driver
sudo modinfo i915 | grep -E "enable_guc|enable_fbc|enable_psr"
echo "Active Command Line:"
cat /proc/cmdline | grep -o "i915[^ ]*" || echo "No custom i915 parameters found in boot options."

echo -e "\n--- 3. GUC/HUC FIRMWARE STATUS ---"
# GuC/HuC are vital for efficient memory and power management on Tiger Lake
sudo dmesg | grep -iE "guc|huc" | grep "version" || echo "GuC/HuC firmware not loaded or not logged."

echo -e "\n--- 4. MEMORY APERTURE (VRAM) ---"
# This shows the 'stolen' memory and the total aperture size
sudo grep -E "stolen|aperture" /sys/kernel/debug/dri/0/i915_capabilities 2>/dev/null || \
sudo dmesg | grep -i "memory" | grep "i915"

echo -e "\n--- 5. GPU MEMORY PRESSURE (Current) ---"
# Check how much memory the GPU has mapped right now
if command -v intel_gpu_top &> /dev/null; then
    echo "intel-gpu-tools found. Displaying snap of IMC (Integrated Memory Controller):"
    sudo timeout -s SIGINT 2s intel_gpu_top | grep -A 5 "IMC"
else
    echo "intel-gpu-tools not installed. Run 'sudo pacman -S intel-gpu-tools' for real-time monitoring."
fi
