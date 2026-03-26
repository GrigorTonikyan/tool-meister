Installation
============

Prerequisites
------------

Linux Limits Manager is designed for Linux systems only and requires:

* Python 3.6 or later
* A terminal with support for curses (most Linux terminals)
* Root privileges for modifying system limits

Standard Installation
-------------------

You can install the Linux Limits Manager using pip:

.. code-block:: bash

    pip install limits-manager

From Source
----------

To install from source:

1. Clone the repository:

   .. code-block:: bash

      git clone https://github.com/username/limits-manager.git
      cd limits-manager

2. Install the package:

   .. code-block:: bash

      pip install .

   For development installation:

   .. code-block:: bash

      pip install -e ".[dev]"

This will install all dependencies and make the `limits-manager` command available in your system.