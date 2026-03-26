#!/usr/bin/env python3
"""Test runner script for the fdlimit package."""

import subprocess
import sys
from pathlib import Path

PROJECT_ROOT = Path(__file__).parent.parent.absolute()


def run_with_coverage():
    """Run tests with coverage reporting."""
    args = [
        "pytest",
        "--cov=fdlimit",
        "--cov-report=term",
    ] + sys.argv[1:]

    return subprocess.call(args)


def run_full_suite():
    """Run the full test suite with comprehensive checks and reporting."""
    args = [
        "pytest",
        "--cov=fdlimit",
        "--cov-report=term",
        "--cov-report=html",
        "--verbose",
    ] + sys.argv[1:]

    return subprocess.call(args)


def run_lint():
    """Run all linting and code quality checks."""
    print("Running ruff (linting)...")
    # Use Python module execution for Ruff
    ruff_result = subprocess.call(
        [sys.executable, "-m", "ruff", "check", "src", "tests", "scripts"]
    )

    print("\nRunning mypy (type checking)...")
    mypy_result = subprocess.call(["mypy", "src"])

    results = {
        "ruff": ruff_result,
        "mypy": mypy_result,
    }

    print("\n=== Linting Results ===")
    all_passed = True
    for check, result in results.items():
        status = "✅ Passed" if result == 0 else "❌ Failed"
        print(f"{check}: {status}")
        if result != 0:
            all_passed = False

    return 0 if all_passed else 1


def run_format():
    """Format code with ruff."""
    print("Formatting code with ruff...")
    # Use Python module execution for Ruff
    result = subprocess.call(
        [sys.executable, "-m", "ruff", "format", "src", "tests", "scripts"]
    )

    if result == 0:
        print("\n✅ Code formatting successful!")
    else:
        print("\n❌ Code formatting failed!")

    return result


if __name__ == "__main__":
    sys.exit(run_full_suite())
