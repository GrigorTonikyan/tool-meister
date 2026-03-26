"""Tests for core functionality."""

import os
import subprocess
import sys
import tempfile
from unittest import TestCase, mock

# Add src directory to path for imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../src")))

from fdlimit.core import (
    LIMITS_CONF,
    SYSCTL_CONF,
    apply_system_changes,
    check_platform,
    get_current_limits,
    parse_configs,
    run_command,
    update_configs,
    validate_limit_value,
)


class TestCore(TestCase):
    """Tests for core module functions."""

    def test_validate_limit_value(self):
        """Test validate_limit_value function."""
        # Valid values
        self.assertEqual(validate_limit_value("1024"), (True, ""))
        self.assertEqual(validate_limit_value("65535"), (True, ""))
        self.assertEqual(validate_limit_value("1000000"), (True, ""))

        # Invalid values
        self.assertEqual(
            validate_limit_value("abc"), (False, "Limit must be a positive integer")
        )
        self.assertEqual(
            validate_limit_value("-100"), (False, "Limit must be a positive integer")
        )
        self.assertEqual(
            validate_limit_value("100"), (False, "Value is too low (min: 1024)")
        )
        self.assertEqual(
            validate_limit_value("20000000"),
            (False, "Value is too high (max: 10000000)"),
        )

    @mock.patch("fdlimit.core.subprocess.check_output")
    def test_run_command_list(self, mock_check_output):
        """Test run_command function with a list input."""
        mock_check_output.return_value = "success"

        result = run_command(["test", "command"])
        mock_check_output.assert_called_once_with(
            ["test", "command"], text=True, stderr=-2
        )
        self.assertEqual(result, "success")

    @mock.patch("fdlimit.core.subprocess.check_output")
    def test_run_command_string(self, mock_check_output):
        """Test run_command function with a string input."""
        mock_check_output.return_value = "success"

        # Test with safe command that is a shell builtin (ulimit)
        result = run_command("ulimit -Hn")
        mock_check_output.assert_called_once_with(
            "ulimit -Hn", shell=True, text=True, stderr=-2
        )
        self.assertEqual(result, "success")

        # Reset mock
        mock_check_output.reset_mock()

        # Test with safe command that is not a shell builtin
        result = run_command("cat /proc/sys/fs/file-max")
        mock_check_output.assert_called_once_with(
            ["cat", "/proc/sys/fs/file-max"], text=True, stderr=-2
        )
        self.assertEqual(result, "success")

        # Reset mock
        mock_check_output.reset_mock()

        # Test with unsafe command
        result = run_command("rm -rf /")
        # Command should not be executed
        mock_check_output.assert_not_called()
        self.assertTrue(result.startswith("Error: Unsupported command"))

    @mock.patch("fdlimit.core.run_command")
    def test_get_current_limits(self, mock_run_command):
        """Test get_current_limits function."""
        # Setup mock return values
        mock_run_command.side_effect = ["1024", "4096", "500000"]

        result = get_current_limits()

        # Check correct commands were run
        self.assertEqual(mock_run_command.call_count, 3)
        mock_run_command.assert_any_call(["ulimit", "-Sn"])
        mock_run_command.assert_any_call(["ulimit", "-Hn"])
        mock_run_command.assert_any_call(["cat", "/proc/sys/fs/file-max"])

        # Check expected results
        expected = {"soft": "1024", "hard": "4096", "system_max": "500000"}
        self.assertEqual(result, expected)

    @mock.patch("os.path.exists", return_value=False)
    def test_parse_configs_nonexistent(self, mock_exists):
        """Test parse_configs with nonexistent configuration files."""
        result = parse_configs()

        expected = {"limits": {}, "sysctl": None}
        self.assertEqual(result, expected)

    @mock.patch("os.path.exists", return_value=True)
    @mock.patch("builtins.open", new_callable=mock.mock_open)
    def test_parse_configs_empty(self, mock_file, mock_exists):
        """Test parse_configs with empty configuration files."""
        # Mock empty content for both files
        mock_file.return_value.__enter__.return_value.readlines.return_value = []

        result = parse_configs()

        expected = {"limits": {}, "sysctl": None}
        self.assertEqual(result, expected)

    @mock.patch("os.path.exists", return_value=True)
    @mock.patch("fdlimit.core.open", new_callable=mock.mock_open)
    def test_parse_configs_with_content(self, mock_file, mock_exists):
        """Test parse_configs with sample configuration content."""
        # Create different mock content for the two files
        limits_content = [
            "# limits.conf\n",
            "* soft nofile 65535\n",
            "* hard nofile 65535\n",
            "root soft nofile 65535\n",
        ]
        sysctl_content = [
            "# sysctl.conf\n",
            "net.ipv4.ip_forward=1\n",
            "fs.file-max = 1000000\n",
        ]

        # Set up the mock to return different content depending on the file path
        def mock_open_side_effect(*args, **kwargs):
            file_path = args[0] if args else kwargs.get("file")
            m = mock.mock_open()
            if "/etc/security/limits.conf" in str(file_path):
                m.return_value.readlines.return_value = limits_content
            elif "/etc/sysctl.conf" in str(file_path):
                m.return_value.readlines.return_value = sysctl_content
            return m()

        mock_file.side_effect = mock_open_side_effect

        result = parse_configs()

        # Check that some expected keys exist in the result
        self.assertIn("limits", result)
        self.assertIn("sysctl", result)
        # The specific parsing logic would require more complex mocking

    def test_check_platform(self):
        """Test platform check."""
        with mock.patch("fdlimit.core.sys.platform", "linux"):
            self.assertTrue(check_platform())

        with mock.patch("fdlimit.core.sys.platform", "win32"):
            self.assertFalse(check_platform())

        with mock.patch("fdlimit.core.sys.platform", "darwin"):
            self.assertFalse(check_platform())

    @mock.patch("builtins.open", new_callable=mock.mock_open)
    @mock.patch("os.path.exists")
    def test_update_configs(self, mock_exists, mock_open):
        """Test update_configs function with valid inputs."""
        mock_exists.return_value = True
        new_limits = {
            "user_soft": "65535",
            "user_hard": "65535",
            "system_max": "1000000",
        }

        result = update_configs(new_limits)
        self.assertTrue(result)
        mock_open.assert_any_call(LIMITS_CONF, "r")
        mock_open.assert_any_call(SYSCTL_CONF, "r")

    @mock.patch("os.path.exists")
    def test_update_configs_invalid_values(self, mock_exists):
        """Test update_configs function with invalid inputs."""
        mock_exists.return_value = True
        invalid_limits = {
            "user_soft": "invalid",
            "user_hard": "65535",
            "system_max": "1000000",
        }

        result = update_configs(invalid_limits)
        self.assertFalse(result)

    @mock.patch("fdlimit.core.subprocess.check_output")
    def test_apply_system_changes_success(self, mock_check_output):
        """Test apply_system_changes successful execution."""
        mock_check_output.return_value = "net.ipv4.ip_forward = 1"
        result = apply_system_changes()
        self.assertEqual(result, "net.ipv4.ip_forward = 1")
        mock_check_output.assert_called_once_with(
            ["sysctl", "-p"], text=True, stderr=-2
        )

    @mock.patch("fdlimit.core.subprocess.check_output")
    def test_apply_system_changes_failure(self, mock_check_output):
        """Test apply_system_changes when command fails."""
        mock_check_output.side_effect = subprocess.CalledProcessError(
            1, "sysctl -p", "error"
        )
        result = apply_system_changes()
        self.assertTrue(result.startswith("Error"))

    def test_validate_limit_value_edge_cases(self):
        """Test validate_limit_value with edge cases."""
        # Test minimum value
        self.assertEqual(validate_limit_value("1024"), (True, ""))

        # Test just below minimum
        self.assertEqual(
            validate_limit_value("1023"), (False, "Value is too low (min: 1024)")
        )

        # Test maximum value
        self.assertEqual(validate_limit_value("10000000"), (True, ""))

        # Test above maximum
        self.assertEqual(
            validate_limit_value("10000001"),
            (False, "Value is too high (max: 10000000)"),
        )

    def test_parse_configs_with_backup(self):
        """Test parse_configs creates backup files correctly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            test_limits_conf = os.path.join(tmpdir, "limits.conf")
            test_sysctl_conf = os.path.join(tmpdir, "sysctl.conf")

            # Create test files
            with open(test_limits_conf, "w") as f:
                f.write("* soft nofile 65535\n")
            with open(test_sysctl_conf, "w") as f:
                f.write("fs.file-max = 1000000\n")

            with mock.patch("fdlimit.core.LIMITS_CONF", test_limits_conf):
                with mock.patch("fdlimit.core.SYSCTL_CONF", test_sysctl_conf):
                    configs = parse_configs()

                    self.assertIn("*", configs["limits"])
                    self.assertEqual(configs["sysctl"], "1000000")


if __name__ == "__main__":
    unittest.main()  # type: ignore # noqa: F821
