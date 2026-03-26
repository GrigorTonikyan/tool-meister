Usage
=====

Command Line Usage
----------------

After installation, you can run the limits manager with:

.. code-block:: bash

    limits-manager

For full functionality, including the ability to modify system limits, run with sudo:

.. code-block:: bash

    sudo limits-manager

Command-Line Arguments
--------------------

.. code-block:: bash

    limits-manager --help
    limits-manager --version
    limits-manager --no-ui  # Non-interactive mode (not implemented yet)

User Interface
------------

The TUI (Text User Interface) provides an interactive way to manage your system's file descriptor limits.

Main Screen
^^^^^^^^^^

The main screen displays:

1. **Current Session Limits**: The limits active in your current session
   - Soft limit: The current limit enforced by the kernel
   - Hard limit: The maximum value that can be set for the soft limit
   - System Max: The total number of file descriptors the system can use

2. **Configured Limits**: The limits set in configuration files
   - User limits: Values from `/etc/security/limits.conf`
   - System Max: Value from `/etc/sysctl.conf`

3. **Options Menu**:
   - Apply Suggested Limits: Sets recommended values (65535/1000000)
   - Custom Limits: Set your own values
   - Reload Configurations: Apply changes from config files
   - Exit: Exit the program

Navigation
^^^^^^^^^

- Use the arrow keys (↑/↓) to navigate the menu
- Press Enter to select an option
- Follow on-screen instructions for input fields

Understanding Linux File Descriptor Limits
---------------------------------------

Linux file descriptor limits control how many files a process can open simultaneously:

- **Soft Limit**: The default limit applied to processes. Can be increased up to the hard limit.

- **Hard Limit**: The maximum a user can set their soft limit to without root privileges.

- **System-wide Limit**: The total number of file descriptors the kernel will allocate.

Configuration Files
-----------------

The tool manages these configuration files:

- `/etc/security/limits.conf`: Controls per-user file descriptor limits
- `/etc/sysctl.conf`: Controls system-wide file descriptor limits

After applying changes, you'll need to log out and back in (or reboot) for user limits to take effect. System-wide changes are applied immediately with `sysctl -p`.