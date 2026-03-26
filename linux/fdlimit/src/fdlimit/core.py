"""Core functionality for file descriptor limits management.

This module contains the core functions for interacting with system limits
and configuration files.
"""

import os
import re
import subprocess
import sys
from typing import Dict, Optional, Tuple, Union

# Config file paths
LIMITS_CONF = "/etc/security/limits.conf"
SYSCTL_CONF = "/etc/sysctl.conf"


def request_sudo() -> bool:
    """Request sudo privileges if not already running as root.

    Returns:
        True if already root or successfully got sudo, False otherwise
    """
    if os.geteuid() == 0:
        return True

    try:
        args = [sys.executable] + sys.argv
        subprocess.check_call(["sudo", "-v"])  # Check if we have sudo access
        os.execvp("sudo", ["sudo"] + args)
    except subprocess.CalledProcessError:
        return False
    except OSError:
        return False


def run_command(cmd: str) -> str:
    """Execute shell command and return output.

    Args:
        cmd: The shell command to execute

    Returns:
        The command output as a string
    """
    try:
        # Handle shell builtins like ulimit which aren't standalone executables
        if isinstance(cmd, list) and cmd[0] == "ulimit":
            return subprocess.check_output(
                f"ulimit {' '.join(cmd[1:])}",
                shell=True,
                text=True,
                stderr=subprocess.STDOUT,
            )
        # Using a list of arguments is safer than shell=True
        elif isinstance(cmd, list):
            return subprocess.check_output(cmd, text=True, stderr=subprocess.STDOUT)
        else:
            # For backwards compatibility, but with safer execution
            # Only allow specific safe commands
            safe_commands = ["ulimit", "cat", "sysctl"]
            cmd_parts = cmd.split()
            if not cmd_parts or cmd_parts[0] not in safe_commands:
                return f"Error: Unsupported command {cmd_parts[0] if cmd_parts else ''}"

            # Handle shell builtins
            if cmd_parts[0] == "ulimit":
                return subprocess.check_output(
                    cmd, shell=True, text=True, stderr=subprocess.STDOUT
                )

            return subprocess.check_output(
                cmd_parts, text=True, stderr=subprocess.STDOUT
            )
    except subprocess.CalledProcessError as e:
        return f"Error: {e.output}"


def validate_limit_value(value: str) -> Tuple[bool, str]:
    """Validate that a limit value is a positive integer.

    Args:
        value: The limit value to validate

    Returns:
        Tuple of (is_valid, error_message)
    """
    # Check if the value is a positive integer
    if not re.match(r"^\d+$", value):
        return False, "Limit must be a positive integer"

    # Check for reasonable range
    try:
        num_value = int(value)
        if num_value < 1024:
            return False, "Value is too low (min: 1024)"
        if num_value > 10000000:
            return False, "Value is too high (max: 10000000)"
        return True, ""
    except ValueError:
        return False, "Invalid numeric value"


def get_current_limits() -> Dict[str, str]:
    """Get current session limits.

    Returns:
        Dictionary with soft, hard and system_max limits
    """
    return {
        "soft": run_command(["ulimit", "-Sn"]).strip(),
        "hard": run_command(["ulimit", "-Hn"]).strip(),
        "system_max": run_command(["cat", "/proc/sys/fs/file-max"]).strip(),
    }


def parse_configs() -> Dict[str, Union[Dict[str, str], Optional[str]]]:
    """Parse configuration files for existing limits.

    Returns:
        Dictionary with parsed configuration values
    """
    configs: Dict[str, Union[Dict[str, str], Optional[str]]] = {
        "limits": {},
        "sysctl": None,
    }

    try:
        if os.path.exists(LIMITS_CONF):
            with open(LIMITS_CONF) as f:
                for line in f:
                    if line.strip().startswith(("*", "root")) and "nofile" in line:
                        parts = line.split()
                        if len(parts) >= 4 and parts[1] in ["soft", "hard"]:
                            configs["limits"][parts[0]] = parts[3]
    except (OSError, PermissionError) as e:
        # Handle specific file access errors
        print(f"Error reading {LIMITS_CONF}: {e}")

    try:
        if os.path.exists(SYSCTL_CONF):
            with open(SYSCTL_CONF) as f:
                for line in f:
                    if line.strip().startswith("fs.file-max"):
                        parts = line.split("=", 1)
                        if len(parts) == 2:
                            configs["sysctl"] = parts[1].strip()
                            break
    except (OSError, PermissionError) as e:
        # Handle specific file access errors
        print(f"Error reading {SYSCTL_CONF}: {e}")

    return configs


def update_configs(new_limits: Dict[str, str]) -> bool:
    """Update configuration files with new limits.

    Args:
        new_limits: Dictionary containing user_soft, user_hard, and system_max limits

    Returns:
        True if update was successful, False otherwise
    """
    # Validate input values
    for key, value in new_limits.items():
        is_valid, error = validate_limit_value(value)
        if not is_valid:
            print(f"Invalid {key} value: {error}")
            return False

    # Create backup of config files before modification
    try:
        if os.path.exists(LIMITS_CONF):
            with open(LIMITS_CONF) as src, open(f"{LIMITS_CONF}.bak", "w") as dst:
                dst.write(src.read())
    except (OSError, PermissionError) as e:
        print(f"Error creating backup of {LIMITS_CONF}: {e}")
        return False

    try:
        if os.path.exists(SYSCTL_CONF):
            with open(SYSCTL_CONF) as src, open(f"{SYSCTL_CONF}.bak", "w") as dst:
                dst.write(src.read())
    except (OSError, PermissionError) as e:
        print(f"Error creating backup of {SYSCTL_CONF}: {e}")
        return False

    # Update limits.conf
    lines = []
    try:
        if os.path.exists(LIMITS_CONF):
            with open(LIMITS_CONF) as f:
                lines = f.readlines()
        else:
            print(f"Creating new {LIMITS_CONF} file")
            lines = ["# File descriptor limits added by limits-manager\n"]
    except (OSError, PermissionError) as e:
        print(f"Error reading {LIMITS_CONF}: {e}")
        return False

    # The rest of the update_configs function remains the same
    new_lines = []
    found = {"* soft": False, "* hard": False, "root soft": False, "root hard": False}

    for line in lines:
        for key in found:
            if line.strip().startswith(key) and "nofile" in line:
                if key == "* soft":
                    new_lines.append(f"* soft nofile {new_limits['user_soft']}\n")
                elif key == "* hard":
                    new_lines.append(f"* hard nofile {new_limits['user_hard']}\n")
                elif key == "root soft":
                    new_lines.append(f"root soft nofile {new_limits['user_soft']}\n")
                elif key == "root hard":
                    new_lines.append(f"root hard nofile {new_limits['user_hard']}\n")
                found[key] = True
                break
        else:
            new_lines.append(line)

    if not found["* soft"]:
        new_lines.append(f"* soft nofile {new_limits['user_soft']}\n")
    if not found["* hard"]:
        new_lines.append(f"* hard nofile {new_limits['user_hard']}\n")
    if not found["root soft"]:
        new_lines.append(f"root soft nofile {new_limits['user_soft']}\n")
    if not found["root hard"]:
        new_lines.append(f"root hard nofile {new_limits['user_hard']}\n")

    try:
        with open(LIMITS_CONF, "w") as f:
            f.writelines(new_lines)
    except (OSError, PermissionError) as e:
        print(f"Error writing to {LIMITS_CONF}: {e}")
        return False

    # Update sysctl.conf
    lines = []
    try:
        if os.path.exists(SYSCTL_CONF):
            with open(SYSCTL_CONF) as f:
                lines = f.readlines()
        else:
            print(f"Creating new {SYSCTL_CONF} file")
            lines = ["# System limits added by limits-manager\n"]
    except (OSError, PermissionError) as e:
        print(f"Error reading {SYSCTL_CONF}: {e}")
        return False

    found = False
    for i, line in enumerate(lines):
        if line.strip().startswith("fs.file-max"):
            lines[i] = f"fs.file-max = {new_limits['system_max']}\n"
            found = True
            break

    if not found:
        lines.append(f"\nfs.file-max = {new_limits['system_max']}\n")

    try:
        with open(SYSCTL_CONF, "w") as f:
            f.writelines(lines)
        return True
    except (OSError, PermissionError) as e:
        print(f"Error writing to {SYSCTL_CONF}: {e}")
        return False


def apply_system_changes() -> str:
    """Apply system changes by reloading configurations.

    Returns:
        Result message from executing the command
    """
    return run_command(["sysctl", "-p"])


def check_platform() -> bool:
    """Check if running on a Linux platform.

    Returns:
        True if running on Linux, False otherwise
    """
    return sys.platform.startswith("linux")
