#!/bin/bash

# --- Global Debug Mode Variable ---
DEBUG_MODE=0

# --- Debug Echo Function ---
debug_echo() {
    if [[ "${DEBUG_MODE:-0}" -eq 1 ]]; then
        echo -e "${YELLOW}DEBUG: $*${RESET}" >&2 # Output debug to stderr
    fi
    return 0
}

debug_echo "Script started."

# --- Initial Dependency Check for JQ ---
if ! command -v jq &>/dev/null; then
    echo "Error: 'jq' is not installed. This script requires 'jq' to parse configuration files." >&2
    echo "Please install 'jq' (e.g., 'sudo pacman -S jq' or your system's equivalent) and try again." >&2
    exit 1
fi
debug_echo "jq found."

# --- Global Configuration Variables ---
declare CONFIG_FILE="config.jsonc"
declare -A APP_CONFIG
declare -a MODULES
declare -A MODULE_CONFIGS
declare -A MODULE_MENUS
declare -A MODULE_COMMANDS
declare anim_delay_sec anim_step_size
declare GLOBAL_ACTION_STATUS=""

has_gum=$(command -v gum &>/dev/null && echo 1 || echo 0)
has_fzf=$(command -v fzf &>/dev/null && echo 1 || echo 0)

RESET='\e[0m' BOLD='\e[1m' INVERT='\e[7m' CYAN='\e[36m' RED='\e[31m'
GREEN='\e[32m' YELLOW='\e[33m' BLUE='\e[34m' WHITE='\e[37m'

# --- JSONC Processing Functions ---
_strip_jsonc_comments() {
    local content="$1"
    content=$(echo "$content" | awk '
        BEGIN { RS = "/\\*"; ORS = ""; }
        NR == 1 { print $0; next }
        { idx = index($0, "*/"); if (idx > 0) { print substr($0, idx + 2); } }
    ')
    content=$(echo "$content" | sed -e 's%^[[:space:]]*//.*%%' -e 's%\s\+//.*%%')
    echo "$content"
    return 0
}

_get_json_value() {
    local json_content="$1"
    local jq_path="$2"
    local default_value="${3:-}"
    local value
    local jq_exit_code=0

    value=$(echo "$json_content" | jq -r "$jq_path" 2>/dev/null)
    jq_exit_code=$?

    if [[ $jq_exit_code -ne 0 ]]; then
        debug_echo "_get_json_value: jq query '$jq_path' failed with exit code $jq_exit_code. Using default: '$default_value'"
        echo "$default_value"
        return 0
    fi

    if [[ "$value" == "null" ]] || [[ -z "$value" ]]; then
        echo "$default_value"
    else
        echo "$value"
    fi
    return 0
}

# --- Config Loading Functions ---
load_app_config() {
    debug_echo "load_app_config called with $1"
    local config_path="$1"
    if [[ ! -f "$config_path" ]]; then
        echo -e "${RED}FATAL: Main configuration file '$config_path' not found.${RESET}" >&2
        exit 1
    fi

    local raw_config_content stripped_config_content
    raw_config_content=$(cat "$config_path")
    debug_echo "Raw config content read from '$config_path'."
    stripped_config_content=$(_strip_jsonc_comments "$raw_config_content")
    debug_echo "Config content stripped."

    local jq_test_output
    if ! jq_test_output=$(echo "$stripped_config_content" | jq -e . 2>&1); then
        echo -e "${RED}FATAL: Invalid JSON in '$config_path' after stripping comments.${RESET}" >&2
        echo -e "${RED}jq error output:\n${RESET}${jq_test_output}" >&2
        exit 1
    fi
    debug_echo "JSON in '$config_path' is valid according to jq -e."

    # Read app settings
    APP_CONFIG[appName]=$(_get_json_value "$stripped_config_content" ".appSettings.appName" "Arch Tool Meister")
    APP_CONFIG[version]=$(_get_json_value "$stripped_config_content" ".appSettings.version" "2.0.0")
    APP_CONFIG[modulesDir]=$(_get_json_value "$stripped_config_content" ".appSettings.modulesDir" "modules")
    APP_CONFIG[animSteps]=$(_get_json_value "$stripped_config_content" ".appSettings.animation.steps" "10")
    APP_CONFIG[animDelayMs]=$(_get_json_value "$stripped_config_content" ".appSettings.animation.delayMs" "320")
    APP_CONFIG[mainMenuFile]=$(_get_json_value "$stripped_config_content" ".menuPaths.main" "main_menu.jsonc")

    # Calculate animation delay in seconds
    anim_delay_sec=$(echo "scale=3; ${APP_CONFIG[animDelayMs]} / 1000" | bc)
    anim_step_size=$(echo "scale=3; 100 / ${APP_CONFIG[animSteps]}" | bc | sed 's/\.0*$//')

    debug_echo "App config loaded: Name=${APP_CONFIG[appName]}, Version=${APP_CONFIG[version]}"
    debug_echo "Modules directory: ${APP_CONFIG[modulesDir]}"
}

# --- Module Discovery and Loading ---
discover_modules() {
    local modules_dir="${APP_CONFIG[modulesDir]}"
    debug_echo "Discovering modules in $modules_dir"

    if [[ ! -d "$modules_dir" ]]; then
        echo -e "${YELLOW}Warning: Modules directory '$modules_dir' not found. Creating it now.${RESET}" >&2
        mkdir -p "$modules_dir"
        return 0
    fi

    local module_count=0
    while IFS= read -r module_dir; do
        local module_name=$(basename "$module_dir")
        if [[ -f "$module_dir/config.jsonc" ]]; then
            MODULES+=("$module_name")
            debug_echo "Found module: $module_name"
            module_count=$((module_count + 1))
        fi
    done < <(find "$modules_dir" -mindepth 1 -maxdepth 1 -type d)

    debug_echo "Discovered $module_count modules"
}

load_module() {
    local module_name="$1"
    local module_dir="${APP_CONFIG[modulesDir]}/$module_name"
    debug_echo "Loading module: $module_name from $module_dir"

    # Load module config
    local config_path="$module_dir/config.jsonc"
    if [[ ! -f "$config_path" ]]; then
        debug_echo "Module $module_name has no config.jsonc, skipping"
        return 1
    fi

    local raw_config_content stripped_config_content
    raw_config_content=$(cat "$config_path")
    stripped_config_content=$(_strip_jsonc_comments "$raw_config_content")

    local jq_test_output
    if ! jq_test_output=$(echo "$stripped_config_content" | jq -e . 2>&1); then
        echo -e "${YELLOW}Warning: Invalid JSON in module config '$config_path', skipping module.${RESET}" >&2
        return 1
    fi

    # Read module config
    local enabled=$(_get_json_value "$stripped_config_content" ".enabled" "true")
    if [[ "$enabled" != "true" ]]; then
        debug_echo "Module $module_name is disabled, skipping"
        return 0
    fi

    # Store module config in associative array
    MODULE_CONFIGS[$module_name]="$stripped_config_content"

    # Load module menu
    local menu_path="$module_dir/menu.jsonc"
    if [[ -f "$menu_path" ]]; then
        local raw_menu_content stripped_menu_content
        raw_menu_content=$(cat "$menu_path")
        stripped_menu_content=$(_strip_jsonc_comments "$raw_menu_content")

        if jq_test_output=$(echo "$stripped_menu_content" | jq -e . 2>&1); then
            MODULE_MENUS[$module_name]="$stripped_menu_content"
            debug_echo "Loaded menu for module $module_name"
        else
            echo -e "${YELLOW}Warning: Invalid JSON in module menu '$menu_path'.${RESET}" >&2
        fi
    else
        debug_echo "Module $module_name has no menu.jsonc"
    fi

    # Load module commands
    local commands_path="$module_dir/commands.jsonc"
    if [[ -f "$commands_path" ]]; then
        local raw_commands_content stripped_commands_content
        raw_commands_content=$(cat "$commands_path")
        stripped_commands_content=$(_strip_jsonc_comments "$raw_commands_content")

        if jq_test_output=$(echo "$stripped_commands_content" | jq -e . 2>&1); then
            MODULE_COMMANDS[$module_name]="$stripped_commands_content"
            debug_echo "Loaded commands for module $module_name"
        else
            echo -e "${YELLOW}Warning: Invalid JSON in module commands '$commands_path'.${RESET}" >&2
        fi
    else
        debug_echo "Module $module_name has no commands.jsonc"
    fi

    # Register module functions
    if [[ -n "${MODULE_COMMANDS[$module_name]}" ]]; then
        register_module_functions "$module_name"
    fi

    debug_echo "Module $module_name loaded successfully"
    return 0
}

register_module_functions() {
    local module_name="$1"
    debug_echo "Registering functions for module: $module_name"

    local functions_json="${MODULE_COMMANDS[$module_name]}"
    local functions_count=$(echo "$functions_json" | jq -r '.functions | keys | length')

    if [[ -z "$functions_count" || "$functions_count" == "null" || "$functions_count" -eq 0 ]]; then
        debug_echo "No functions found in module $module_name"
        return 0
    fi

    for func_name in $(echo "$functions_json" | jq -r '.functions | keys[]'); do
        local func_code=$(echo "$functions_json" | jq -r ".functions.\"$func_name\".code")
        if [[ -n "$func_code" && "$func_code" != "null" ]]; then
            # Create function dynamically
            eval "function ${module_name}__${func_name}() {
        local MODULE_NAME=\"$module_name\"
        local MODULE_CONFIG
        declare -A MODULE_CONFIG
        
        # Parse module config into associative array for easy access
        local config_json=\"\${MODULE_CONFIGS[\$MODULE_NAME]}\"
        local settings_keys=\$(echo \"\$config_json\" | jq -r '.settings | keys[]' 2>/dev/null)
        for key in \$settings_keys; do
          local value=\$(echo \"\$config_json\" | jq -r \".settings.\\\"\$key\\\"\" 2>/dev/null)
          if [[ \"\$value\" != \"null\" && -n \"\$value\" ]]; then
            MODULE_CONFIG[\$key]=\"\$value\"
            # For nested objects, create flattened keys
            if [[ \$(echo \"\$value\" | jq -e 'type == \"object\"' 2>/dev/null) == \"true\" ]]; then
              local nested_keys=\$(echo \"\$value\" | jq -r 'keys[]' 2>/dev/null)
              for nested_key in \$nested_keys; do
                local nested_value=\$(echo \"\$value\" | jq -r \".\\\"\$nested_key\\\"\" 2>/dev/null)
                if [[ \"\$nested_value\" != \"null\" && -n \"\$nested_value\" ]]; then
                  MODULE_CONFIG[\$key.\$nested_key]=\"\$nested_value\"
                fi
              done
            fi
          fi
        done
        
        $func_code
      }"
            debug_echo "Registered function: ${module_name}__${func_name}"
        fi
    done

    debug_echo "Registered $(echo "$functions_json" | jq -r '.functions | keys | length') functions for module $module_name"
}

module_execute_command() {
    local module_name="$1"
    local command_name="$2"
    shift 2 # Remove module_name and command_name from args

    local commands_json="${MODULE_COMMANDS[$module_name]}"
    if [[ -z "$commands_json" ]]; then
        echo -e "${RED}ERROR: No commands defined for module $module_name${RESET}" >&2
        return 1
    fi

    local function_name=$(echo "$commands_json" | jq -r ".commands.\"$command_name\".function")
    if [[ -z "$function_name" || "$function_name" == "null" ]]; then
        echo -e "${RED}ERROR: Command '$command_name' not found in module $module_name${RESET}" >&2
        return 1
    fi

    # Check dependencies
    local deps=$(echo "$commands_json" | jq -r ".commands.\"$command_name\".dependencies[]" 2>/dev/null)
    if [[ -n "$deps" && "$deps" != "null" ]]; then
        for dep in $deps; do
            if ! command -v "$dep" &>/dev/null; then
                echo -e "${YELLOW}WARNING: Required dependency '$dep' not found.${RESET}" >&2
                echo -e "${YELLOW}This command may not work properly.${RESET}" >&2
                echo -e "${YELLOW}Please install '$dep' and try again.${RESET}" >&2
                read -rp "Press Enter to continue anyway or Ctrl+C to abort..."
            fi
        done
    fi

    # Get additional args from command definition
    local cmd_args=$(echo "$commands_json" | jq -r ".commands.\"$command_name\".args[]" 2>/dev/null)
    local all_args=($cmd_args "$@")

    # Call the module function
    debug_echo "Executing module command: ${module_name}__${function_name} ${all_args[*]}"
    "${module_name}__${function_name}" "${all_args[@]}"
    return $?
}

# --- Main Menu Handling ---
load_main_menu() {
    local menu_file="${APP_CONFIG[mainMenuFile]}"
    debug_echo "Loading main menu from $menu_file"

    if [[ ! -f "$menu_file" ]]; then
        echo -e "${RED}FATAL: Main menu file '$menu_file' not found.${RESET}" >&2
        exit 1
    fi

    local raw_menu_content stripped_menu_content
    raw_menu_content=$(cat "$menu_file")
    stripped_menu_content=$(_strip_jsonc_comments "$raw_menu_content")

    local jq_test_output
    if ! jq_test_output=$(echo "$stripped_menu_content" | jq -e . 2>&1); then
        echo -e "${RED}FATAL: Invalid JSON in '$menu_file' after stripping comments.${RESET}" >&2
        echo -e "${RED}jq error output:\n${RESET}${jq_test_output}" >&2
        exit 1
    fi

    local is_dynamic=$(_get_json_value "$stripped_menu_content" ".dynamicMenu" "false")
    if [[ "$is_dynamic" == "true" ]]; then
        # Generate dynamic main menu with module entries
        local insert_position=$(echo "$stripped_menu_content" | jq -r '.options | length - 1')

        for module_name in "${MODULES[@]}"; do
            local module_config="${MODULE_CONFIGS[$module_name]}"
            if [[ -z "$module_config" ]]; then
                continue
            fi

            local menu_title=$(_get_json_value "$module_config" ".mainMenuEntry" "")
            if [[ -z "$menu_title" || "$menu_title" == "null" ]]; then
                menu_title=$(_get_json_value "$module_config" ".menuTitle" "$module_name")
            fi

            if [[ -z "$menu_title" || "$menu_title" == "null" ]]; then
                continue # Skip modules without a menu title
            fi

            # Add module entry to main menu
            stripped_menu_content=$(echo "$stripped_menu_content" | jq --arg title "$menu_title" --arg module "$module_name" \
                ".options = (.options[0:$insert_position] + [{\"text\": \$title, \"type\": \"moduleMenu\", \"module\": \$module}] + .options[$insert_position:])")
        done

        debug_echo "Generated dynamic main menu with module entries"
    fi

    echo "$stripped_menu_content"
}

display_menu() {
    local menu_json="$1"
    local selected_idx="${2:-0}"
    local title module_name

    title=$(_get_json_value "$menu_json" ".title" "Menu")
    module_name=$(_get_json_value "$menu_json" ".module" "")

    # Display VSCode versions if this is the VSCode module menu
    local header_info=""
    if [[ "$module_name" == "vscode" ]]; then
        local installed_stable_ver installed_insiders_ver online_stable_ver online_insiders_ver

        # Get installed versions
        if [[ -x "/usr/local/bin/code" ]] && command -v /usr/local/bin/code &>/dev/null; then
            installed_stable_ver=$(/usr/local/bin/code --version | head -n 1 2>/dev/null || echo "Not found")
        else
            installed_stable_ver="Not installed"
        fi

        if [[ -x "/usr/local/bin/code-insiders" ]] && command -v /usr/local/bin/code-insiders &>/dev/null; then
            installed_insiders_ver=$(/usr/local/bin/code-insiders --version | head -n 1 2>/dev/null || echo "Not found")
        else
            installed_insiders_ver="Not installed"
        fi

        # Try to get online versions (quick non-blocking curl)
        online_stable_ver=$(curl -s -m 2 -L -I -o /dev/null -w '%{url_effective}' \
            "https://update.code.visualstudio.com/latest/linux-x64/stable" 2>/dev/null |
            grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "Unknown")

        online_insiders_ver=$(curl -s -m 2 -L -I -o /dev/null -w '%{url_effective}' \
            "https://update.code.visualstudio.com/latest/linux-x64/insider" 2>/dev/null |
            grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "Unknown")

        # Create header with version info
        local stable_label insiders_label
        stable_label=$(_get_json_value "${MODULE_CONFIGS[$module_name]}" ".settings.stable.label" "VS Code Stable")
        insiders_label=$(_get_json_value "${MODULE_CONFIGS[$module_name]}" ".settings.insiders.label" "VS Code Insiders")

        header_info=$(
            cat <<EOF
${BOLD}${BLUE}${title}${RESET}

${CYAN}Currently Installed:${RESET}
$(printf "  ${WHITE}%-10s: %s${RESET}" "$stable_label" "$installed_stable_ver")
$(printf "  ${WHITE}%-10s: %s${RESET}" "$insiders_label" "$installed_insiders_ver")

${CYAN}Available Online:${RESET}
$(printf "  ${WHITE}%-10s: %s${RESET}" "$stable_label" "$online_stable_ver")
$(printf "  ${WHITE}%-10s: %s${RESET}" "$insiders_label" "$online_insiders_ver")
EOF
        )
    else
        header_info="${BOLD}${BLUE}${title}${RESET}"
    fi

    echo -e "$header_info\n"

    echo -e "${BLUE}┌──────────────────────────────────┐${RESET}"

    local options_count=$(_get_json_value "$menu_json" ".options | length" "0")
    for ((i = 0; i < options_count; i++)); do
        local text=$(_get_json_value "$menu_json" ".options[$i].text" "Option $((i + 1))")
        local display_text="$((i + 1))) $text"

        if [[ $i -eq $selected_idx ]]; then
            echo -e "${BLUE}│${RESET} ${INVERT}${display_text}${RESET}"
        else
            echo -e "${BLUE}│${RESET} ${display_text}"
        fi
    done

    echo -e "${BLUE}└──────────────────────────────────┘${RESET}"
    echo -e "Use ↑↓, ⏎, or numbers to select. 0 to go back/exit."
}

process_menu_selection() {
    local menu_json="$1"
    local selection="$2"
    local module_name=$(_get_json_value "$menu_json" ".module" "")

    local options_count=$(_get_json_value "$menu_json" ".options | length" "0")
    if [[ "$selection" -lt 1 || "$selection" -gt "$options_count" ]]; then
        echo -e "${RED}Invalid selection. Please try again.${RESET}"
        return 1
    fi

    local idx=$((selection - 1))
    local option_type=$(_get_json_value "$menu_json" ".options[$idx].type" "")
    local option_text=$(_get_json_value "$menu_json" ".options[$idx].text" "Selected option")

    case "$option_type" in
    "scriptFunction")
        local func_name=$(_get_json_value "$menu_json" ".options[$idx].functionName" "")
        if [[ -z "$func_name" ]]; then
            echo -e "${RED}ERROR: No function name specified for menu option.${RESET}" >&2
            return 1
        fi

        # For system commands defined in the old way
        if declare -f "$func_name" >/dev/null; then
            "$func_name"
            local result=$?
        else
            # Check if this is a module command
            local found=0
            for module in "${MODULES[@]}"; do
                local commands_json="${MODULE_COMMANDS[$module]}"
                if [[ -n "$commands_json" ]] && [[ $(echo "$commands_json" | jq -e ".commands.\"$func_name\"") != "null" ]]; then
                    module_execute_command "$module" "$func_name"
                    local result=$?
                    found=1
                    break
                fi
            done

            if [[ $found -eq 0 ]]; then
                echo -e "${RED}ERROR: Function '$func_name' not found.${RESET}" >&2
                return 1
            fi
        fi

        # If this is a VSCode operation, inform that versions will refresh
        if [[ "$module_name" == "vscode" ]] && [[ "$func_name" == "deploy_"* || "$func_name" == "uninstall_"* ]]; then
            echo -e "${GREEN}VSCode operation finished. Versions will refresh on next menu display.${RESET}"
        fi

        return $result
        ;;

    "moduleMenu")
        local target_module=$(_get_json_value "$menu_json" ".options[$idx].module" "")
        if [[ -z "$target_module" ]]; then
            echo -e "${RED}ERROR: No module specified for menu option.${RESET}" >&2
            return 1
        fi

        if [[ -n "${MODULE_MENUS[$target_module]}" ]]; then
            local sub_menu="${MODULE_MENUS[$target_module]}"
            # Add module name to menu JSON for context
            sub_menu=$(echo "$sub_menu" | jq --arg module "$target_module" '. + {module: $module}')
            display_and_process_menu "$sub_menu"
            return 0 # Always return success after processing a submenu
        else
            echo -e "${RED}ERROR: Menu not found for module '$target_module'.${RESET}" >&2
            return 1
        fi
        ;;

    "return")
        echo -e "${BLUE}Returning to previous menu...${RESET}"
        return 0
        ;;

    "exit")
        echo -e "${BLUE}Exiting...${RESET}"
        exit 0
        ;;

    *)
        echo -e "${RED}ERROR: Unknown menu option type: $option_type${RESET}" >&2
        return 1
        ;;
    esac
}

# Loading animation function
loading() {
    if [[ $has_gum -eq 1 ]]; then
        gum spin --title "Loading..." -- sleep "$anim_delay_sec"
    else
        echo -n "Loading"
        for ((i = 0; i < ${APP_CONFIG[animSteps]}; i++)); do
            sleep "$anim_step_size"
            echo -n "."
        done
        echo
    fi
}

display_and_process_menu() {
    local menu_json="$1"
    local selected_idx=0
    local key_nav key_nav2

    while true; do
        clear
        display_menu "$menu_json" "$selected_idx"

        IFS= read -rsn1 key_nav
        if [[ $key_nav == $'\x1b' ]]; then
            # Arrow key navigation
            read -rsn2 -t 0.1 key_nav2 || true
            key_nav+=$key_nav2
            local options_count=$(_get_json_value "$menu_json" ".options | length" "0")

            case $key_nav in
            $'\x1b[A') # Up arrow
                ((selected_idx = (selected_idx - 1 + options_count) % options_count))
                ;;
            $'\x1b[B') # Down arrow
                ((selected_idx = (selected_idx + 1) % options_count))
                ;;
            esac
        elif [[ $key_nav == "" ]]; then
            # Enter key pressed
            local selection=$((selected_idx + 1))
            echo -e "\n${GREEN}Selected option $selection${RESET}"
            loading
            process_menu_selection "$menu_json" "$selection"

            # Check if we should return (handled in process_menu_selection)
            if [[ $? -eq 0 ]]; then
                continue
            fi

            echo ""
            read -rp "Press Enter to continue..." _
        elif [[ $key_nav == "0" ]]; then
            # '0' key to exit or go back
            local module_name=$(_get_json_value "$menu_json" ".module" "")
            if [[ -n "$module_name" ]]; then
                # This is a module menu, return to main menu
                return 0
            else
                # This is the main menu, exit the program
                echo -e "${BLUE}Exiting...${RESET}"
                exit 0
            fi
        elif [[ "$key_nav" =~ ^[0-9]$ ]]; then
            # Number keys for direct selection
            local options_count=$(_get_json_value "$menu_json" ".options | length" "0")
            local selection=$((key_nav))

            if [[ "$selection" -ge 1 && "$selection" -le "$options_count" ]]; then
                echo -e "\n${GREEN}Selected option $selection${RESET}"
                loading
                process_menu_selection "$menu_json" "$selection"

                # Check if we should return (handled in process_menu_selection)
                if [[ $? -eq 0 ]]; then
                    continue
                fi

                echo ""
                read -rp "Press Enter to continue..." _
            else
                echo -e "${RED}Invalid selection. Please enter a number between 1 and $options_count${RESET}"
                sleep 1
            fi
        fi
    done
}

# --- System Command Functions ---
run_uname_a() {
    echo -e "${BLUE}System Information:${RESET}"
    uname -a
}

run_pwd() {
    echo -e "${BLUE}Current Directory:${RESET}"
    pwd
}

run_ls_lh() {
    echo -e "${BLUE}Files in Current Directory:${RESET}"
    ls -lh
}

# --- CLI Arguments Processing ---
parse_cli_args() {
    debug_echo "parse_cli_args called with: $*"
    local direct_action_executed=0

    while [[ $# -gt 0 ]]; do
        case "$1" in
        --debug)
            debug_echo "Debug flag already processed"
            shift
            ;;
        --module)
            if [[ -n "$2" ]]; then
                local module="$2"
                shift 2
                if [[ -n "$1" ]]; then
                    local command="$1"
                    shift
                    echo -e "${BLUE}Executing module command: $module $command${RESET}"
                    
                    # Need to load config and modules first for CLI commands
                    load_app_config "$CONFIG_FILE"
                    discover_modules
                    
                    # Load the specific module
                    if load_module "$module"; then
                        module_execute_command "$module" "$command" "$@"
                    else
                        echo -e "${RED}ERROR: Failed to load module $module${RESET}" >&2
                        exit 1
                    fi
                    direct_action_executed=1
                    break # Stop processing args as the rest will be passed to the command
                else
                    echo -e "${RED}ERROR: No command specified for module $module${RESET}" >&2
                    exit 1
                fi
            else
                echo -e "${RED}ERROR: No module specified${RESET}" >&2
                exit 1
            fi
            ;;
        --list-modules)
            load_app_config "$CONFIG_FILE"
            discover_modules
            echo -e "${BLUE}Available modules:${RESET}"
            for module in "${MODULES[@]}"; do
                echo " - $module"
            done
            direct_action_executed=1
            shift
            ;;
        *)
            echo -e "${RED}ERROR: Unknown option: $1${RESET}" >&2
            echo "Usage: $0 [--debug] [--module <module-name> <command-name> [args...]] [--list-modules]" >&2
            exit 1
            ;;
        esac
    done

    if [[ $direct_action_executed -eq 1 ]]; then
        exit 0
    fi
    debug_echo "parse_cli_args finished, no direct action taken that caused exit."
}

# --- Main Entry Point ---
# Standalone debug flag check (before load_app_config)
for arg_scan in "$@"; do
    if [[ "$arg_scan" == "--debug" ]]; then
        DEBUG_MODE=1
        debug_echo "Debug mode ENABLED by initial scan."
        break
    fi
done

main() {
    parse_cli_args "$@"

    # Check for bc dependency before loading config that uses it
    if ! command -v bc &>/dev/null; then
        echo "Error: 'bc' is not installed. This script requires 'bc' for calculations." >&2
        echo "Please install 'bc' (e.g., 'sudo pacman -S bc' or your system's equivalent) and try again." >&2
        exit 1
    fi

    load_app_config "$CONFIG_FILE"
    discover_modules

    # Load all enabled modules
    for module in "${MODULES[@]}"; do
        load_module "$module"
    done

    # Display main menu
    local main_menu_json
    main_menu_json=$(load_main_menu)
    display_and_process_menu "$main_menu_json"
}

main "$@"
