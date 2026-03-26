"""User interface module for the file descriptor limits manager.

This module contains the TUI (Text User Interface) for the file descriptor limits manager.
"""

import curses
from curses import textpad
from os import geteuid
from typing import Dict, Optional

from fdlimit.core import (
    apply_system_changes,
    get_current_limits,
    parse_configs,
    request_sudo,
    update_configs,
    validate_limit_value,
)

# Constants for minimum safe values
MIN_FD_LIMIT = 1024
RECOMMENDED_SOFT = "65535"
RECOMMENDED_HARD = "65535"
RECOMMENDED_SYSTEM = "1000000"


def draw_menu(stdscr) -> None:
    """Main menu drawing function.

    Args:
        stdscr: The curses standard screen object
    """
    curses.curs_set(0)
    stdscr.clear()
    h, w = stdscr.getmaxyx()
    current_row = 0
    needs_reboot = False
    suggested = {
        "user_soft": RECOMMENDED_SOFT,
        "user_hard": RECOMMENDED_HARD,
        "system_max": RECOMMENDED_SYSTEM,
    }

    # Set up colors
    curses.start_color()
    curses.init_pair(1, curses.COLOR_BLACK, curses.COLOR_WHITE)
    curses.init_pair(2, curses.COLOR_GREEN, curses.COLOR_BLACK)
    curses.init_pair(3, curses.COLOR_YELLOW, curses.COLOR_BLACK)
    curses.init_pair(4, curses.COLOR_RED, curses.COLOR_BLACK)

    while True:
        stdscr.clear()
        status_msg = " [ROOT]" if geteuid() == 0 else " [NOT ROOT]"
        status_color = curses.color_pair(2) if geteuid() == 0 else curses.color_pair(4)

        # Display title with status
        stdscr.addstr(0, 0, "Linux File Limit Manager", curses.A_BOLD)
        stdscr.addstr(
            0, len("Linux File Limit Manager"), status_msg, status_color | curses.A_BOLD
        )

        # Display current limits
        limits = get_current_limits()
        stdscr.addstr(2, 0, "Current Session Limits:", curses.A_UNDERLINE)
        stdscr.addstr(3, 2, f"Soft: {limits['soft']}")
        stdscr.addstr(4, 2, f"Hard: {limits['hard']}")
        stdscr.addstr(5, 2, f"System Max: {limits['system_max']}")

        # Display configured limits
        configs = parse_configs()
        stdscr.addstr(7, 0, "Configured Limits:", curses.A_UNDERLINE)
        stdscr.addstr(8, 2, f"User: {configs['limits'].get('*', 'Not set')}")
        stdscr.addstr(9, 2, f"Root: {configs['limits'].get('root', 'Not set')}")
        stdscr.addstr(10, 2, f"System Max: {configs['sysctl'] or 'Not set'}")

        # Menu options
        menu = [
            f"Apply Suggested Limits ({RECOMMENDED_SOFT}/{RECOMMENDED_SYSTEM})",
            "Custom Limits",
            "Reload Configurations",
            "Exit",
        ]

        stdscr.addstr(12, 0, "Options:", curses.A_UNDERLINE)

        for idx, item in enumerate(menu):
            x = w // 2 - len(item) // 2
            y = 13 + idx
            if idx == current_row:
                stdscr.attron(curses.color_pair(1))
                stdscr.addstr(y, x, item)
                stdscr.attroff(curses.color_pair(1))
            else:
                stdscr.addstr(y, x, item)

        # Help text
        stdscr.addstr(h - 2, 0, "Use ↑↓ to navigate, Enter to select", curses.A_DIM)

        key = stdscr.getch()

        if key == curses.KEY_UP and current_row > 0:
            current_row -= 1
        elif key == curses.KEY_DOWN and current_row < len(menu) - 1:
            current_row += 1
        elif key == curses.KEY_ENTER or key in [10, 13]:
            if menu[current_row] == "Exit":
                break
            elif (
                menu[current_row]
                == f"Apply Suggested Limits ({RECOMMENDED_SOFT}/{RECOMMENDED_SYSTEM})"
            ):
                if geteuid() != 0:
                    show_message(
                        stdscr,
                        "Run with sudo to modify system files!",
                        curses.color_pair(4) | curses.A_BOLD,
                    )
                elif update_configs(suggested):
                    apply_system_changes()
                    needs_reboot = True
                    show_message(
                        stdscr,
                        "Limits updated! Log out/in to apply.",
                        curses.color_pair(2) | curses.A_BOLD,
                    )
                else:
                    show_message(
                        stdscr,
                        "Failed to update limits. Check permissions and values.",
                        curses.color_pair(4) | curses.A_BOLD,
                    )
            elif menu[current_row] == "Custom Limits":
                custom_limits = show_custom_dialog(stdscr)
                if custom_limits and geteuid() == 0:
                    if update_configs(custom_limits):
                        apply_system_changes()
                        needs_reboot = True
                        show_message(
                            stdscr,
                            "Custom limits applied!",
                            curses.color_pair(2) | curses.A_BOLD,
                        )
                    else:
                        show_message(
                            stdscr,
                            "Failed to apply custom limits. Check values.",
                            curses.color_pair(4) | curses.A_BOLD,
                        )
                elif custom_limits and geteuid() != 0:
                    show_message(
                        stdscr,
                        "Root privileges required!",
                        curses.color_pair(4) | curses.A_BOLD,
                    )
            elif menu[current_row] == "Reload Configurations":
                result = apply_system_changes()
                if "Error" in result:
                    show_message(
                        stdscr,
                        f"Error reloading configurations: {result}",
                        curses.color_pair(4) | curses.A_BOLD,
                    )
                else:
                    show_message(
                        stdscr,
                        "System configurations reloaded!",
                        curses.color_pair(2) | curses.A_BOLD,
                    )

        stdscr.refresh()

    if needs_reboot:
        # Final message after exiting curses mode
        return "System limits updated. Reboot or re-login to apply changes.\n"


def show_custom_dialog(stdscr) -> Optional[Dict[str, str]]:
    """Show custom limits input dialog.

    Args:
        stdscr: The curses standard screen object

    Returns:
        Dictionary with user_soft, user_hard, and system_max limits or None if cancelled
    """
    h, w = stdscr.getmaxyx()
    win_h, win_w = 12, 60  # Made dialog larger for validation messages
    win = curses.newwin(win_h, win_w, h // 2 - win_h // 2, w // 2 - win_w // 2)
    textpad.rectangle(win, 0, 0, win_h - 1, win_w - 1)
    win.addstr(1, 2, "Enter Custom Limits:", curses.A_BOLD)
    win.addstr(
        2,
        2,
        f"(Minimum recommended: {MIN_FD_LIMIT}, suggested: {RECOMMENDED_SOFT})",
        curses.A_DIM,
    )
    win.addstr(
        win_h - 2,
        2,
        "Press Tab to move between fields, Enter to submit, ESC to cancel",
        curses.A_DIM,
    )

    fields = [
        ("User Soft Limit:", 4, ""),
        ("User Hard Limit:", 5, ""),
        ("System Max Limit:", 6, ""),
    ]

    current_field = 0
    inputs = ["", "", ""]
    error_msg = ""

    while True:
        # Display all fields
        for idx, (label, y, _) in enumerate(fields):
            win.addstr(y, 2, label + " " * (win_w - len(label) - 4))
            if idx == current_field:
                win.attron(curses.A_REVERSE)
            win.addstr(y, len(label) + 2, inputs[idx] + " " * (20 - len(inputs[idx])))
            if idx == current_field:
                win.attroff(curses.A_REVERSE)

        # Display error message if any
        if error_msg:
            win.addstr(8, 2, " " * (win_w - 4))  # Clear previous error
            win.addstr(8, 2, error_msg, curses.color_pair(4) | curses.A_BOLD)
        else:
            win.addstr(8, 2, " " * (win_w - 4))  # Clear line

        win.refresh()
        key = win.getch()

        if key == 9:  # Tab key
            current_field = (current_field + 1) % len(fields)
            error_msg = ""
        elif key == curses.KEY_ENTER or key in [10, 13]:
            # Validate inputs
            valid = True

            # Check if all fields have values
            if not all(inputs):
                error_msg = "All fields are required!"
                valid = False

            # Check that soft ≤ hard
            if valid and inputs[0] and inputs[1]:
                try:
                    if int(inputs[0]) > int(inputs[1]):
                        error_msg = "Soft limit cannot be larger than hard limit"
                        valid = False
                except ValueError:
                    error_msg = "Limits must be numeric values"
                    valid = False

            # Validate each value
            if valid:
                for i, value in enumerate(inputs):
                    is_valid, err_msg = validate_limit_value(value)
                    if not is_valid:
                        error_msg = err_msg
                        valid = False
                        break

            if valid:
                return {
                    "user_soft": inputs[0],
                    "user_hard": inputs[1],
                    "system_max": inputs[2],
                }

        elif key == 27:  # ESC key
            return None
        elif key == curses.KEY_BACKSPACE or key == 127:  # Backspace
            if inputs[current_field]:
                inputs[current_field] = inputs[current_field][:-1]
                error_msg = ""
        elif 48 <= key <= 57:  # Numbers only
            if len(inputs[current_field]) < 20:
                inputs[current_field] += chr(key)
                error_msg = ""
        else:
            error_msg = "Only numeric input is allowed"


def show_message(stdscr, message: str, attr=curses.A_BOLD) -> None:
    """Show a message at the bottom of the screen.

    Args:
        stdscr: The curses standard screen object
        message: The message to display
        attr: The curses attributes to apply to the message
    """
    h, _ = stdscr.getmaxyx()
    # Clear the line first
    stdscr.addstr(h - 1, 0, " " * (stdscr.getmaxyx()[1] - 1))
    stdscr.addstr(h - 1, 0, message, attr)
    stdscr.refresh()
    stdscr.getch()


def run_ui() -> None:
    """Run the user interface with proper curses setup/teardown."""
    try:
        result = curses.wrapper(draw_menu)
        if result:
            print(result)
    except KeyboardInterrupt:
        print("\nProgram interrupted by user.")
    except Exception as e:
        print(f"An error occurred: {e}")
        if isinstance(e, PermissionError):
            print("Requesting sudo privileges...")
            if request_sudo():
                return  # The program will restart with sudo
