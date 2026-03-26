#!/usr/bin/env python3
"""Example script demonstrating how to use fdlimit programmatically.

This script shows how to use the fdlimit package as a library rather than
through its command-line interface.
"""

import os
import sys

# Add the parent directory to the path so we can import the package
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from fdlimit.core import (
    apply_system_changes,
    check_platform,
    get_current_limits,
    parse_configs,
    update_configs,
)


def display_limits():
    """Display the current limits."""
    print("== Current Session Limits ==")
    limits = get_current_limits()
    print(f"Soft Limit: {limits['soft']}")
    print(f"Hard Limit: {limits['hard']}")
    print(f"System Max: {limits['system_max']}")

    print("\n== Configured Limits ==")
    configs = parse_configs()
    print(f"User limits: {configs['limits'].get('*', 'Not set')}")
    print(f"Root limits: {configs['limits'].get('root', 'Not set')}")
    print(f"System max: {configs['sysctl'] or 'Not set'}")


def set_recommended_limits():
    """Set recommended limits for high-performance applications."""
    if os.geteuid() != 0:
        print("Error: Root privileges required to set limits!")
        return False

    print("Setting recommended limits (65535/1000000)...")
    new_limits = {"user_soft": "65535", "user_hard": "65535", "system_max": "1000000"}

    if update_configs(new_limits):
        print("Configuration files updated successfully!")
        print("Applying system changes...")
        result = apply_system_changes()
        print(f"Result: {result}")
        print("Note: User limits require logout/login to take effect.")
        return True
    else:
        print("Failed to update configuration files!")
        return False


def main():
    """Main example function."""
    print("Linux File Descriptor Limits Manager - Programmatic Usage Example")
    print("================================================\n")

    if not check_platform():
        print("Error: This tool is designed for Linux systems only.")
        return 1

    # Display current limits
    display_limits()

    # Ask if user wants to set recommended limits
    print("\nDo you want to set recommended limits? (requires root) [y/N] ", end="")
    response = input().lower()

    if response == "y" or response == "yes":
        set_recommended_limits()

    return 0


if __name__ == "__main__":
    sys.exit(main())
