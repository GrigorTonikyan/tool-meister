"""Command-line interface for the file descriptor limits manager.

This module provides the entry point for the command-line interface.
"""

import argparse
import sys
from os import geteuid
from typing import Dict, List, Optional

from fdlimit.core import (
    apply_system_changes,
    check_platform,
    get_current_limits,
    parse_configs,
    request_sudo,
    update_configs,
)
from fdlimit.ui import RECOMMENDED_HARD, RECOMMENDED_SOFT, RECOMMENDED_SYSTEM, run_ui


def parse_args(args: Optional[List[str]] = None) -> argparse.Namespace:
    """Parse command line arguments.

    Args:
        args: Command-line arguments to parse (defaults to sys.argv[1:])

    Returns:
        Parsed arguments namespace
    """
    parser = argparse.ArgumentParser(
        description="Manage Linux file descriptor limits with a TUI interface."
    )
    parser.add_argument("--version", action="store_true", help="Show program version")
    parser.add_argument(
        "--no-ui",
        action="store_true",
        help="Run in non-interactive mode",
    )
    parser.add_argument(
        "--apply-recommended",
        action="store_true",
        help="Apply recommended limits (65535/1000000) in non-interactive mode",
    )
    parser.add_argument(
        "--set-limits",
        nargs=3,
        metavar=("SOFT", "HARD", "SYSTEM_MAX"),
        help="Set custom limits in non-interactive mode (3 values: soft hard system_max)",
    )
    parser.add_argument(
        "--show-current",
        action="store_true",
        help="Show current limits and exit",
    )

    return parser.parse_args(args)


def show_current_limits() -> None:
    """Display current system limits."""
    limits = get_current_limits()
    configs = parse_configs()

    print("=== Current Session Limits ===")
    print(f"Soft Limit: {limits['soft']}")
    print(f"Hard Limit: {limits['hard']}")
    print(f"System Max: {limits['system_max']}")

    print("\n=== Configured Limits ===")
    print(f"User (*) Limit: {configs['limits'].get('*', 'Not set')}")
    print(f"Root Limit: {configs['limits'].get('root', 'Not set')}")
    print(f"System Max: {configs['sysctl'] or 'Not set'}")


def apply_limits(limits: Dict[str, str]) -> bool:
    """Apply the specified limits.

    Args:
        limits: Dictionary with user_soft, user_hard, and system_max values

    Returns:
        True if successful, False otherwise
    """
    if geteuid() != 0:
        print("Root privileges required. Requesting sudo access...")
        if not request_sudo():
            print("Failed to obtain root privileges.")
            return False
        return True  # If request_sudo succeeds, it will restart the program with sudo

    if update_configs(limits):
        result = apply_system_changes()
        print(f"Applied system changes: {result}")
        print("Note: Log out and back in for user limits to take effect.")
        return True
    else:
        print("Error: Failed to update configuration files!")
        return False


def main(args: Optional[List[str]] = None) -> int:
    """Main entry point for the command-line interface.

    Args:
        args: Command-line arguments (defaults to sys.argv[1:])

    Returns:
        Exit code
    """
    parsed_args = parse_args(args)

    if parsed_args.version:
        from fdlimit import __version__

        print(f"fdlimit version {__version__}")
        return 0

    if not check_platform():
        print("This tool is designed for Linux systems only.")
        return 1

    if parsed_args.show_current:
        show_current_limits()
        return 0

    if parsed_args.no_ui:
        if parsed_args.apply_recommended:
            limits = {
                "user_soft": RECOMMENDED_SOFT,
                "user_hard": RECOMMENDED_HARD,
                "system_max": RECOMMENDED_SYSTEM,
            }
            if apply_limits(limits):
                return 0
            else:
                return 1

        elif parsed_args.set_limits:
            soft, hard, sys_max = parsed_args.set_limits
            limits = {
                "user_soft": soft,
                "user_hard": hard,
                "system_max": sys_max,
            }
            if apply_limits(limits):
                return 0
            else:
                return 1
        else:
            print(
                "Error: In non-interactive mode, you must specify either --apply-recommended or --set-limits."
            )
            print("Run 'fdlimit --help' for usage information.")
            return 1

    try:
        run_ui()
        return 0
    except PermissionError:
        print("Root privileges required. Requesting sudo access...")
        if request_sudo():
            return 0  # Program will restart with sudo
        else:
            print("Failed to obtain root privileges.")
            return 1
    except Exception as e:
        print(f"Error: {e}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
