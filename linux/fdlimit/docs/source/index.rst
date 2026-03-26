Linux Limits Manager
===================

A Python utility for managing Linux file descriptor limits through a user-friendly TUI interface.

.. toctree::
   :maxdepth: 2
   :caption: Contents:
   
   introduction
   installation
   usage
   api
   contributing

Introduction
-----------

Linux Limits Manager helps system administrators and developers easily manage file descriptor limits on Linux systems. 
File descriptors are crucial system resources that represent open files, network sockets, and other I/O resources.

For high-performance applications, databases, or services handling many concurrent connections, proper configuration 
of these limits is essential to prevent "Too many open files" errors and ensure optimal performance.

Features
--------

* View current session file descriptor limits
* Apply recommended limits for high-performance applications
* Set custom limits for user and system-wide file descriptors 
* Manage both session and persistent limits
* Easy to use TUI (Text User Interface)

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`