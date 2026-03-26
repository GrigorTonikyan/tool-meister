Introduction
============

What is Linux Limits Manager?
---------------------------

Linux Limits Manager is a specialized tool for managing file descriptor limits on Linux systems. It provides an intuitive Text User Interface (TUI) that allows system administrators and developers to easily view and modify both user-level and system-wide file descriptor limits.

Why File Descriptor Limits Matter
-------------------------------

File descriptors are used by the Linux kernel to represent open files, network sockets, pipes, and other I/O resources. Each process has a limited number of file descriptors it can use, and the system as a whole has an overall limit.

When these limits are set too low, applications can encounter the dreaded "Too many open files" error, especially in:

* High-traffic web servers
* Database systems
* Container hosts
* Applications handling many concurrent connections
* Development environments running multiple services

Key Concepts
----------

Linux has several different types of file descriptor limits:

1. **Soft Limit**: The practical limit enforced for a process. A process can increase its soft limit up to the hard limit.

2. **Hard Limit**: The maximum value the soft limit can be set to without requiring root privileges.

3. **System-Wide Limit**: The total number of file descriptors the kernel can allocate across all processes.

Configuration Files
-----------------

These limits are configured in different files:

* `/etc/security/limits.conf`: Controls per-user soft and hard limits
* `/etc/sysctl.conf`: Controls system-wide limits

How Linux Limits Manager Helps
----------------------------

Linux Limits Manager provides:

1. **Easy Visibility**: Quickly see current and configured limits in one interface
2. **Simple Management**: Apply recommended limits with a single action
3. **Custom Configuration**: Set precise values for your specific needs
4. **Safe Updates**: Handles the configuration files properly to prevent errors
5. **Immediate Feedback**: See the results of your changes