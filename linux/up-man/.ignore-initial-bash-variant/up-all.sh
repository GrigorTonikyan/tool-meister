#!/usr/bin/env bash

# Set error handling
set -o pipefail # Fail if any command in a pipe fails

# Track overall success
OVERALL_SUCCESS=true

# Configuration file location
CONFIG_FILE="$HOME/.config/up-all.conf"
ALIAS_NAME="${1:-up-all}" # Default alias name if not provided as argument

# Display help
show_help() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --setup-alias      Setup shell alias for this script"
    echo "  --validate         Validate configuration file"
    echo "  --backup           Backup configuration file"
    echo "  --no-alias-setup   Run without offering alias setup"
    echo "  --help             Show this help message"
    echo ""
    echo "Environment:"
    echo "  Config file: $CONFIG_FILE"
    echo ""
}

# Create default config if it doesn't exist
if [ ! -f "$CONFIG_FILE" ]; then
    mkdir -p "$(dirname "$CONFIG_FILE")"
    cat >"$CONFIG_FILE" <<EOL
# Package managers configuration
# Format: ENABLED::COMMAND::NEEDS_SUDO

# System package managers
APT=true::apt update && apt upgrade -y && apt autoremove -y::true
SNAP=true::snap refresh::true
FLATPAK=true::flatpak update -y::false

# Language/tool specific package managers
BREW=true::brew update && brew upgrade && brew cleanup && brew doctor::false
PIP=false::pip list --outdated --format=freeze | grep -v '^\-e' | cut -d = -f 1 | xargs -n1 pip install -U::false
PIP3=false::pip3 list --outdated --format=freeze | grep -v '^\-e' | cut -d = -f 1 | xargs -n1 pip3 install -U::false
NPM=false::npm update -g::false
RUSTUP=false::rustup update::false
CARGO=false::cargo install-update -a::false
GO=false::go get -u all::false
EOL
    echo "Created default configuration at $CONFIG_FILE"
    echo "Edit this file to enable/disable package managers"
fi

# Function to detect available package managers
detect_package_managers() {

    local detected=()

    # Check for package managers not in default config
    if command -v pip &>/dev/null && ! grep -q "^PIP=" "$CONFIG_FILE"; then
        detected+=("PIP")
    fi

    if command -v pip3 &>/dev/null && ! grep -q "^PIP3=" "$CONFIG_FILE"; then
        detected+=("PIP3")
    fi

    if command -v npm &>/dev/null && ! grep -q "^NPM=" "$CONFIG_FILE"; then
        detected+=("NPM")
    fi

    if command -v rustup &>/dev/null && ! grep -q "^RUSTUP=" "$CONFIG_FILE"; then
        detected+=("RUSTUP")
    fi

    if command -v cargo-install-update &>/dev/null && ! grep -q "^CARGO=" "$CONFIG_FILE"; then
        detected+=("CARGO")
    fi

    if command -v go &>/dev/null && ! grep -q "^GO=" "$CONFIG_FILE"; then
        detected+=("GO")
    fi

    # If we detected any new package managers, suggest them
    if [ ${#detected[@]} -gt 0 ]; then
        echo "Detected additional package managers that could be enabled in $CONFIG_FILE:"
        for pm in "${detected[@]}"; do
            echo "- $pm"
        done
        echo "Edit $CONFIG_FILE to enable them."
    fi
}

# Validate configuration file
validate_config() {
    local has_errors=false
    local line_number=0

    echo "Validating configuration in $CONFIG_FILE..."

    # Check if file exists and is readable
    if [ ! -f "$CONFIG_FILE" ] || [ ! -r "$CONFIG_FILE" ]; then
        echo "❌ Config file doesn't exist or is not readable."
        return 1
    fi

    # Check each line
    while IFS= read -r line || [ -n "$line" ]; do
        ((line_number++))
        # Skip comments and empty lines
        [[ "$line" =~ ^#.*$ || -z "$line" ]] && continue

        # Validate format - using :: as separator
        if ! [[ "$line" =~ ^([A-Z0-9_]+)=([^:]*[^[:space:]]+)[[:space:]]*::[[:space:]]*(.+)[[:space:]]*::[[:space:]]*([^:]*[^[:space:]]+)[[:space:]]*$ ]]; then
            echo "❌ Invalid line format at line $line_number: $line"
            echo "   Format should be: NAME=ENABLED::COMMAND::NEEDS_SUDO"
            has_errors=true
            continue
        fi

        # Check enabled value
        local enabled="${BASH_REMATCH[2]// /}"
        if [ "$enabled" != "true" ] && [ "$enabled" != "false" ]; then
            echo "❌ Invalid enabled value at line $line_number: $line (must be 'true' or 'false')"
            has_errors=true
        fi

        # Check needs_sudo value
        local needs_sudo="${BASH_REMATCH[4]// /}"
        if [ "$needs_sudo" != "true" ] && [ "$needs_sudo" != "false" ]; then
            echo "❌ Invalid needs_sudo value at line $line_number: $line (must be 'true' or 'false')"
            has_errors=true
        fi
    done <"$CONFIG_FILE"

    if [ "$has_errors" = "true" ]; then
        echo "⚠️ Configuration has errors. Please fix them before continuing."
        return 1
    else
        echo "✅ Configuration is valid."
        return 0
    fi
}

# Backup configuration file
backup_config() {
    local backup_file="$CONFIG_FILE.backup.$(date +%Y%m%d%H%M%S)"
    cp "$CONFIG_FILE" "$backup_file" && echo "✅ Configuration backed up to $backup_file"
}

# Setup alias in appropriate shell config file
setup_alias() {
    local shell_rc=""
    local alias_name="${2:-$ALIAS_NAME}" # Use second argument if provided, otherwise default

    if [ -n "$ZSH_VERSION" ]; then
        shell_rc="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        if [ -f "$HOME/.bashrc" ]; then
            shell_rc="$HOME/.bashrc"
        elif [ -f "$HOME/.bash_profile" ]; then
            shell_rc="$HOME/.bash_profile"
        fi
    fi

    if [ -n "$shell_rc" ]; then
        # Get absolute path to the script
        local script_path=$(realpath "$0")

        if ! grep -q "alias ${alias_name}=" "$shell_rc"; then
            echo "alias ${alias_name}='${script_path}'" >>"$shell_rc"
            echo "Alias '${alias_name}' added to $shell_rc"
            echo "Restart your shell or run 'source $shell_rc' to use the alias"
        else
            echo "Alias '${alias_name}' already exists in $shell_rc"
        fi
    else
        echo "Could not determine shell configuration file. Please add the alias manually:"
        echo "alias ${alias_name}='$(realpath "$0")'"
    fi
}

# Run updates
run_updates() {
    echo "Starting system update with up-all script..."
    echo "Using configuration from $CONFIG_FILE"
    echo "----------------------------------------"

    local failure_count=0
    local success_count=0
    local line_number=0

    # Read and process each line from config file
    while IFS= read -r line || [ -n "$line" ]; do
        ((line_number++))
        # Skip comments and empty lines
        [[ "$line" =~ ^#.*$ || -z "$line" ]] && continue

        # Parse the line: NAME=enabled::command::needs_sudo
        if [[ "$line" =~ ^([A-Z0-9_]+)=([^:]*[^[:space:]]+)[[:space:]]*::[[:space:]]*(.+)[[:space:]]*::[[:space:]]*([^:]*[^[:space:]]+)[[:space:]]*$ ]]; then
            local name="${BASH_REMATCH[1]}"
            local enabled="${BASH_REMATCH[2]// /}" # Remove any spaces
            local command="${BASH_REMATCH[3]}"
            local needs_sudo="${BASH_REMATCH[4]// /}" # Remove any spaces

            # Skip if not enabled
            [ "$enabled" != "true" ] && continue

            echo "⏳ Updating $name..."

            if [ "$needs_sudo" = "true" ]; then
                sudo bash -c "$command"
            else
                eval "$command"
            fi

            local status=$?
            if [ $status -eq 0 ]; then
                echo "✅ $name update completed successfully."
                ((success_count++))
            else
                echo "❌ $name update failed with status $status."
                OVERALL_SUCCESS=false
                ((failure_count++))
            fi
            echo "----------------------------------------"
        else
            echo "⚠️ Warning: Invalid configuration line at line $line_number: $line"
            echo "   Format should be: NAME=ENABLED::COMMAND::NEEDS_SUDO"
        fi
    done <"$CONFIG_FILE"

    echo "Update summary: $success_count succeeded, $failure_count failed"
    if [ "$OVERALL_SUCCESS" = "true" ]; then
        echo "✅ All updates completed successfully!"
    else
        echo "⚠️ Some updates failed. Check the log above for details."
    fi
}

# Main execution
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    show_help
elif [ "$1" = "--setup-alias" ]; then
    # Pass the second argument (if any) as the alias name
    setup_alias "$1" "$2"
elif [ "$1" = "--validate" ]; then
    validate_config
elif [ "$1" = "--backup" ]; then
    backup_config
else
    detect_package_managers

    # Validate and ask for confirmation if there are errors
    if ! validate_config; then
        read -p "Configuration errors were found. Do you want to continue anyway? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Exiting."
            exit 1
        fi
        echo "Continuing despite configuration errors..."
    fi

    run_updates

    # Offer to setup alias if not already done and not already running through alias
    if [ "$1" != "--no-alias-setup" ]; then
        script_path=$(realpath "$0")
        script_basename=$(basename "$0")

        # Don't prompt if called through alias or if running with correct name
        if [[ "$0" == "$script_path" && "$script_basename" != "$ALIAS_NAME" ]]; then
            read -p "Would you like to setup an alias for this script? (y/n) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                setup_alias
            fi
        fi
    fi
fi
