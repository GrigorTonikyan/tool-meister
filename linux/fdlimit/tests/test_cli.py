"""Tests for the CLI module."""

import os
import sys
import unittest
from unittest import mock

# Add src directory to path for imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../src")))

from fdlimit.cli import apply_limits, main, parse_args, show_current_limits


class TestCLI(unittest.TestCase):
    """Tests for CLI module functions."""

    def test_parse_args_default(self):
        """Test parse_args with default arguments."""
        args = parse_args([])
        self.assertFalse(args.version)
        self.assertFalse(args.no_ui)
        self.assertFalse(args.apply_recommended)
        self.assertFalse(args.show_current)
        self.assertIsNone(args.set_limits)

    def test_parse_args_version(self):
        """Test parse_args with --version argument."""
        args = parse_args(["--version"])
        self.assertTrue(args.version)

    def test_parse_args_no_ui(self):
        """Test parse_args with --no-ui argument."""
        args = parse_args(["--no-ui"])
        self.assertTrue(args.no_ui)

    def test_parse_args_apply_recommended(self):
        """Test parse_args with --apply-recommended argument."""
        args = parse_args(["--no-ui", "--apply-recommended"])
        self.assertTrue(args.no_ui)
        self.assertTrue(args.apply_recommended)

    def test_parse_args_set_limits(self):
        """Test parse_args with --set-limits argument."""
        args = parse_args(["--no-ui", "--set-limits", "2048", "4096", "1000000"])
        self.assertTrue(args.no_ui)
        self.assertEqual(args.set_limits, ["2048", "4096", "1000000"])

    def test_parse_args_show_current(self):
        """Test parse_args with --show-current argument."""
        args = parse_args(["--show-current"])
        self.assertTrue(args.show_current)

    @mock.patch("fdlimit.cli.get_current_limits")
    @mock.patch("fdlimit.cli.parse_configs")
    @mock.patch("builtins.print")
    def test_show_current_limits(self, mock_print, mock_parse_configs, mock_get_limits):
        """Test show_current_limits function."""
        # Set up mocks
        mock_get_limits.return_value = {
            "soft": "1024",
            "hard": "4096",
            "system_max": "500000",
        }
        mock_parse_configs.return_value = {
            "limits": {"*": "65535", "root": "65535"},
            "sysctl": "1000000",
        }

        # Call the function
        show_current_limits()

        # Verify it prints the expected output
        mock_print.assert_any_call("=== Current Session Limits ===")
        mock_print.assert_any_call("Soft Limit: 1024")
        mock_print.assert_any_call("Hard Limit: 4096")
        mock_print.assert_any_call("System Max: 500000")

    @mock.patch("fdlimit.cli.geteuid")
    @mock.patch("fdlimit.cli.update_configs")
    @mock.patch("fdlimit.cli.apply_system_changes")
    @mock.patch("builtins.print")
    def test_apply_limits_success(
        self, mock_print, mock_apply, mock_update, mock_geteuid
    ):
        """Test apply_limits function success case."""
        # Set up mocks
        mock_geteuid.return_value = 0  # Root user
        mock_update.return_value = True
        mock_apply.return_value = "Successfully applied"

        limits = {"user_soft": "65535", "user_hard": "65535", "system_max": "1000000"}
        result = apply_limits(limits)

        self.assertTrue(result)
        mock_update.assert_called_once_with(limits)
        mock_apply.assert_called_once()

    @mock.patch("fdlimit.cli.geteuid")
    @mock.patch("fdlimit.cli.update_configs")
    @mock.patch("builtins.print")
    def test_apply_limits_no_root(self, mock_print, mock_update, mock_geteuid):
        """Test apply_limits function when not root."""
        # Set up mocks
        mock_geteuid.return_value = 1000  # Non-root user

        limits = {"user_soft": "65535", "user_hard": "65535", "system_max": "1000000"}
        result = apply_limits(limits)

        self.assertFalse(result)
        mock_update.assert_not_called()
        mock_print.assert_any_call(
            "Error: Root privileges required to modify system limits!"
        )

    @mock.patch("fdlimit.cli.check_platform")
    def test_main_non_linux_platform(self, mock_check_platform):
        """Test main function on non-Linux platform."""
        mock_check_platform.return_value = False

        exit_code = main([])

        self.assertEqual(exit_code, 1)

    @mock.patch("fdlimit.cli.check_platform")
    @mock.patch("fdlimit.cli.run_ui")
    def test_main_default(self, mock_run_ui, mock_check_platform):
        """Test main function with default arguments."""
        mock_check_platform.return_value = True

        exit_code = main([])

        mock_run_ui.assert_called_once()
        self.assertEqual(exit_code, 0)

    @mock.patch("fdlimit.cli.check_platform")
    @mock.patch("fdlimit.__version__", "0.1.0")  # Mock the correct module
    @mock.patch("builtins.print")
    def test_main_version(self, mock_print, mock_check_platform):
        """Test main function with --version argument."""
        exit_code = main(["--version"])
        mock_print.assert_called_with("fdlimit version 0.1.0")
        self.assertEqual(exit_code, 0)

    @mock.patch("fdlimit.cli.check_platform")
    @mock.patch("fdlimit.cli.show_current_limits")
    def test_main_show_current(self, mock_show_current, mock_check_platform):
        """Test main function with --show-current argument."""
        mock_check_platform.return_value = True

        exit_code = main(["--show-current"])

        mock_show_current.assert_called_once()
        self.assertEqual(exit_code, 0)

    @mock.patch("fdlimit.cli.check_platform")
    @mock.patch("fdlimit.cli.apply_limits")
    def test_main_apply_recommended(self, mock_apply, mock_check_platform):
        """Test main function with --apply-recommended argument."""
        mock_check_platform.return_value = True
        mock_apply.return_value = True

        exit_code = main(["--no-ui", "--apply-recommended"])

        mock_apply.assert_called_once()
        self.assertEqual(exit_code, 0)

    @mock.patch("fdlimit.cli.check_platform")
    @mock.patch("fdlimit.cli.apply_limits")
    def test_main_set_limits(self, mock_apply, mock_check_platform):
        """Test main function with --set-limits argument."""
        mock_check_platform.return_value = True
        mock_apply.return_value = True

        exit_code = main(["--no-ui", "--set-limits", "2048", "4096", "1000000"])

        mock_apply.assert_called_once()
        self.assertEqual(exit_code, 0)

    @mock.patch("fdlimit.cli.check_platform")
    @mock.patch("builtins.print")
    def test_main_no_ui_no_options(self, mock_print, mock_check_platform):
        """Test main function with --no-ui but no other options."""
        mock_check_platform.return_value = True

        exit_code = main(["--no-ui"])

        self.assertEqual(exit_code, 1)
        mock_print.assert_any_call(
            "Error: In non-interactive mode, you must specify either --apply-recommended or --set-limits."
        )


if __name__ == "__main__":
    unittest.main()
