"""Sphinx configuration for fdlimit documentation."""

import os
import sys

# Add the package source directory to the Python path
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', 'src')))

# Project information
project = "fdlimit"
copyright = "2025"
author = "Greg"

# The full version, including alpha/beta/rc tags
from fdlimit import __version__
release = __version__
version = __version__

# General configuration
extensions = [
    "sphinx.ext.autodoc",
    "sphinx.ext.viewcode",
    "sphinx.ext.napoleon",
]

templates_path = ["_templates"]
exclude_patterns = []

# HTML output options
html_theme = "sphinx_rtd_theme"
html_static_path = ["_static"]

# autodoc configuration
autodoc_member_order = "bysource"
autodoc_typehints = "description"
add_module_names = False  # Don't prefix everything with the module name

# napoleon configuration
napoleon_google_docstring = True
napoleon_numpy_docstring = False
napoleon_use_param = True
napoleon_use_rtype = True
napoleon_attr_annotations = True
