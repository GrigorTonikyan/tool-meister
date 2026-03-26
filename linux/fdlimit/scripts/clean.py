#!/usr/bin/env python3
"""Clean script for removing temporary files and build artifacts from the project."""

import os
import shutil
from pathlib import Path

DIRS_TO_CLEAN = [
    "build",
    "dist",
    ".pytest_cache",
    "htmlcov",
    ".coverage",  # .coverage can sometimes be a directory
    ".mypy_cache",
    "__pycache__",
    ".tox",
    ".eggs",
    ".venv",
    "venv",  # Added venv directory
    "_build",
    ".cache",
]

FILES_TO_CLEAN = [
    "*.pyc",
    "*.pyo",
    "*.pyd",
    "*~",
    "*.bak",
    "*.swp",
    ".coverage",  # Added .coverage as a file
    ".coverage.*",
    "coverage.xml",
    ".DS_Store",
    "coverage.json",
    "poetry.lock",  # Added poetry.lock to the clean list
]

PACKAGE_NAME = "fdlimit"
PROJECT_ROOT = Path(__file__).parent.parent.absolute()


def find_directories_to_clean():
    """Find all directories that match patterns in DIRS_TO_CLEAN."""
    dirs_to_remove = []

    # First get direct matches in project root
    for pattern in DIRS_TO_CLEAN:
        path = PROJECT_ROOT / pattern
        if path.exists() and path.is_dir():
            dirs_to_remove.append(path)

    # Then find __pycache__ directories recursively
    for root, dirs, _ in os.walk(PROJECT_ROOT):
        root_path = Path(root)
        # Exclude .git directory from cleaning
        if ".git" in dirs:
            dirs.remove(".git")

        for dir_name in dirs:
            if dir_name == "__pycache__" or dir_name.endswith(".egg-info"):
                dirs_to_remove.append(root_path / dir_name)

    return dirs_to_remove


def find_files_to_clean():
    """Find all files that match patterns in FILES_TO_CLEAN."""
    files_to_remove = []

    # First check for exact files at the root level
    for pattern in FILES_TO_CLEAN:
        if "*" not in pattern and "?" not in pattern:  # Not a wildcard pattern
            exact_file = PROJECT_ROOT / pattern
            if exact_file.exists() and exact_file.is_file():
                files_to_remove.append(exact_file)

    # Then do the full recursive search
    for root, _, files in os.walk(PROJECT_ROOT):
        root_path = Path(root)
        # Skip .git directories
        if ".git" in root:
            continue

        for file_name in files:
            file_path = root_path / file_name
            for pattern in FILES_TO_CLEAN:
                if file_path.match(pattern):
                    files_to_remove.append(file_path)
                    break

    return files_to_remove


def clean():
    """Remove all temporary files and directories."""
    print(f"Cleaning project: {PROJECT_ROOT.name}")

    # Find and remove directories
    dirs_to_remove = find_directories_to_clean()
    for directory in dirs_to_remove:
        try:
            if directory.exists():
                print(f"Removing directory: {directory.relative_to(PROJECT_ROOT)}")
                shutil.rmtree(directory)
        except Exception as e:
            print(f"Error removing {directory}: {e}")

    # Find and remove files
    files_to_remove = find_files_to_clean()
    for file_path in files_to_remove:
        try:
            if file_path.exists():
                print(f"Removing file: {file_path.relative_to(PROJECT_ROOT)}")
                file_path.unlink()
        except Exception as e:
            print(f"Error removing {file_path}: {e}")


def main():
    """Main entry point for the clean script."""
    clean()
    print("Cleaning completed successfully!")


if __name__ == "__main__":
    main()
