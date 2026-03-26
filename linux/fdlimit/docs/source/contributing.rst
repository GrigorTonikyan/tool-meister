Contributing
============

Thank you for considering contributing to Linux Limits Manager! We welcome contributions of all kinds from bug reports to feature suggestions and code contributions.

Development Setup
---------------

1. Clone the repository:

   .. code-block:: bash

       git clone https://github.com/username/limits-manager.git
       cd limits-manager

2. Create a development environment:

   .. code-block:: bash

       # Install with development dependencies
       pip install -e ".[dev]"

Code Standards
------------

This project follows these coding standards:

- Code formatting with Black (line length: 88 characters)
- Import sorting with isort
- Type checking with mypy
- Testing with pytest

Running Tests
------------

Run the tests with:

.. code-block:: bash

    # Run all tests
    pytest

    # Run with coverage report
    pytest --cov=limits_manager

    # Run specific test
    pytest tests/test_core.py

Pull Request Process
------------------

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and ensure they pass
5. Update documentation if necessary
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to your branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

Building Documentation
--------------------

To build the documentation:

.. code-block:: bash

    cd docs
    make html

The generated documentation will be in the `docs/_build/html` directory.