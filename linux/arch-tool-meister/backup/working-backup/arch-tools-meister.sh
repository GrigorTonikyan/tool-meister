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
declare APP_DOWNLOAD_DIR INSTALL_PREFIX ANIM_STEPS ANIM_DELAY_MS
declare VSCODE_STABLE_URL VSCODE_STABLE_DIR_NAME VSCODE_STABLE_SYMLINK_NAME VSCODE_STABLE_LABEL
declare VSCODE_INSIDERS_URL VSCODE_INSIDERS_DIR_NAME VSCODE_INSIDERS_SYMLINK_NAME VSCODE_INSIDERS_LABEL
declare MAIN_MENU_FILE_PATH VSCODE_MENU_FILE_PATH AUR_HELPERS_MENU_FILE_PATH GIT_CONFIG_MENU_FILE_PATH
declare YAY_REPO_URL YAY_CLONE_DIR PARU_REPO_URL PARU_CLONE_DIR
declare anim_delay_sec anim_step_size
declare GLOBAL_ACTION_STATUS=""

has_gum=$(command -v gum &>/dev/null && echo 1 || echo 0)
has_fzf=$(command -v fzf &>/dev/null && echo 1 || echo 0)

RESET='\e[0m' BOLD='\e[1m' INVERT='\e[7m' CYAN='\e[36m' RED='\e[31m'
GREEN='\e[32m' YELLOW='\e[33m' BLUE='\e[34m' WHITE='\e[37m'

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
  if [[ "$DEBUG_MODE" -eq 1 ]]; then
    debug_echo "Stripped config.jsonc content START:"
    echo "$stripped_config_content" >&2
    debug_echo "Stripped config.jsonc content END."
  fi

  local jq_test_output
  if ! jq_test_output=$(echo "$stripped_config_content" | jq -e . 2>&1); then
    echo -e "${RED}FATAL: Invalid JSON in '$config_path' after stripping comments.${RESET}" >&2
    echo -e "${RED}jq error output:\n${RESET}${jq_test_output}" >&2
    exit 1
  fi
  debug_echo "JSON in '$config_path' is valid according to jq -e."

  APP_DOWNLOAD_DIR=$(_get_json_value "$stripped_config_content" ".appSettings.downloadDir" "/tmp/vscode-dl")
  INSTALL_PREFIX=$(_get_json_value "$stripped_config_content" ".appSettings.installPrefix" "/opt")
  ANIM_STEPS=$(_get_json_value "$stripped_config_content" ".appSettings.animation.steps" "10")
  ANIM_DELAY_MS=$(_get_json_value "$stripped_config_content" ".appSettings.animation.delayMs" "320")

  VSCODE_STABLE_URL=$(_get_json_value "$stripped_config_content" ".vscodeConfig.stable.url")
  VSCODE_STABLE_DIR_NAME=$(_get_json_value "$stripped_config_content" ".vscodeConfig.stable.dirName")
  VSCODE_STABLE_SYMLINK_NAME=$(_get_json_value "$stripped_config_content" ".vscodeConfig.stable.symlinkName")
  VSCODE_STABLE_LABEL=$(_get_json_value "$stripped_config_content" ".vscodeConfig.stable.label")

  VSCODE_INSIDERS_URL=$(_get_json_value "$stripped_config_content" ".vscodeConfig.insiders.url")
  VSCODE_INSIDERS_DIR_NAME=$(_get_json_value "$stripped_config_content" ".vscodeConfig.insiders.dirName")
  VSCODE_INSIDERS_SYMLINK_NAME=$(_get_json_value "$stripped_config_content" ".vscodeConfig.insiders.symlinkName")
  VSCODE_INSIDERS_LABEL=$(_get_json_value "$stripped_config_content" ".vscodeConfig.insiders.label")

  MAIN_MENU_FILE_PATH=$(_get_json_value "$stripped_config_content" ".menuPaths.main" "main_menu.jsonc")
  VSCODE_MENU_FILE_PATH=$(_get_json_value "$stripped_config_content" ".menuPaths.vscode" "vscode_menu.jsonc")
  AUR_HELPERS_MENU_FILE_PATH=$(_get_json_value "$stripped_config_content" ".menuPaths.aurHelpers" "aur_helpers.jsonc")
  GIT_CONFIG_MENU_FILE_PATH=$(_get_json_value "$stripped_config_content" ".menuPaths.gitConfig" "git_config.jsonc")

  YAY_REPO_URL=$(_get_json_value "$stripped_config_content" ".aurHelpers.yay.repoUrl" "https://aur.archlinux.org/yay.git")
  YAY_CLONE_DIR=$(_get_json_value "$stripped_config_content" ".aurHelpers.yay.cloneDir" "yay")
  PARU_REPO_URL=$(_get_json_value "$stripped_config_content" ".aurHelpers.paru.repoUrl" "https://aur.archlinux.org/paru-bin.git")
  PARU_CLONE_DIR=$(_get_json_value "$stripped_config_content" ".aurHelpers.paru.cloneDir" "paru-bin")

  debug_echo "APP_DOWNLOAD_DIR='${APP_DOWNLOAD_DIR}'"
  debug_echo "INSTALL_PREFIX='${INSTALL_PREFIX}'"
  debug_echo "VSCODE_STABLE_URL='${VSCODE_STABLE_URL}'"
  debug_echo "MAIN_MENU_FILE_PATH='${MAIN_MENU_FILE_PATH}'"

  if [[ -z "$VSCODE_STABLE_URL" ]] || [[ -z "$VSCODE_INSIDERS_URL" ]]; then
    echo -e "${RED}FATAL: VSCode URLs not found or empty in '$config_path'.${RESET}" >&2
    debug_echo "VSCODE_STABLE_URL value was: '${VSCODE_STABLE_URL}'"
    debug_echo "VSCODE_INSIDERS_URL value was: '${VSCODE_INSIDERS_URL}'"
    exit 1
  fi
  if [[ -z "$MAIN_MENU_FILE_PATH" ]]; then
    echo -e "${RED}FATAL: Main menu file path not found or empty in '$config_path'.${RESET}" >&2
    debug_echo "MAIN_MENU_FILE_PATH value was: '${MAIN_MENU_FILE_PATH}'"
    exit 1
  fi
  debug_echo "load_app_config finished successfully."
}

parse_cli_args() {
  debug_echo "parse_cli_args called with: $*"
  local direct_action_executed=0

  while [[ $# -gt 0 ]]; do
    case "$1" in
    --debug)
      shift
      ;;
    --install-code-stable)
      debug_echo "Direct action: --install-code-stable"
      default_check_dep curl "critical"
      default_check_dep tar "critical"
      echo -e "${BLUE}Direct: Installing Stable...${RESET}"
      deploy_variant "$VSCODE_STABLE_LABEL" "$VSCODE_STABLE_URL" "$VSCODE_STABLE_DIR_NAME" "$VSCODE_STABLE_SYMLINK_NAME"
      direct_action_executed=1
      shift
      ;;
    --install-code-insiders)
      debug_echo "Direct action: --install-code-insiders"
      default_check_dep curl "critical"
      default_check_dep tar "critical"
      echo -e "${BLUE}Direct: Installing Insiders...${RESET}"
      deploy_variant "$VSCODE_INSIDERS_LABEL" "$VSCODE_INSIDERS_URL" "$VSCODE_INSIDERS_DIR_NAME" "$VSCODE_INSIDERS_SYMLINK_NAME"
      direct_action_executed=1
      shift
      ;;
    --install-code-both)
      debug_echo "Direct action: --install-code-both"
      default_check_dep curl "critical"
      default_check_dep tar "critical"
      echo -e "${BLUE}Direct: Installing Both...${RESET}"
      deploy_variant "$VSCODE_STABLE_LABEL" "$VSCODE_STABLE_URL" "$VSCODE_STABLE_DIR_NAME" "$VSCODE_STABLE_SYMLINK_NAME"
      deploy_variant "$VSCODE_INSIDERS_LABEL" "$VSCODE_INSIDERS_URL" "$VSCODE_INSIDERS_DIR_NAME" "$VSCODE_INSIDERS_SYMLINK_NAME"
      direct_action_executed=1
      shift
      ;;
    --install-yay)
      debug_echo "Direct action: --install-yay"
      default_check_dep git "critical"
      echo -e "${BLUE}Direct: Installing Yay...${RESET}"
      install_aur_helper "$YAY_REPO_URL" "$YAY_CLONE_DIR"
      direct_action_executed=1
      shift
      ;;
    --install-paru)
      debug_echo "Direct action: --install-paru"
      default_check_dep git "critical"
      echo -e "${BLUE}Direct: Installing Paru...${RESET}"
      install_aur_helper "$PARU_REPO_URL" "$PARU_CLONE_DIR"
      direct_action_executed=1
      shift
      ;;
    *)
      debug_echo "Ignoring unknown CLI argument: $1"
      shift
      ;;
    esac
  done

  if [[ $direct_action_executed -eq 1 ]]; then
    echo -e "${GREEN}Direct actions complete.${RESET}"
    exit 0
  fi
  debug_echo "parse_cli_args finished, no direct action taken that caused exit."
}

# Standalone debug flag check (before load_app_config and strict mode)
for arg_scan in "$@"; do if [[ "$arg_scan" == "--debug" ]]; then
  DEBUG_MODE=1
  debug_echo "Debug mode ENABLED by initial scan."
  break
fi; done

load_app_config "$CONFIG_FILE"

trap 'debug_echo "EXIT trap triggered. Cleaning up $APP_DOWNLOAD_DIR"; [[ -d "$APP_DOWNLOAD_DIR" ]] && rm -rf "$APP_DOWNLOAD_DIR"' EXIT HUP INT TERM
debug_echo "App config loaded. Trap set using APP_DOWNLOAD_DIR='${APP_DOWNLOAD_DIR}'"

default_check_dep() {
  local dep_name="$1" criticality_level="${2:-critical}"
  if command -v "$dep_name" >/dev/null 2>&1; then
    debug_echo "Dependency '$dep_name' already installed."
    return 0
  fi
  echo -e "${YELLOW}Dependency ${BOLD}${dep_name}${RESET}${YELLOW} not installed.${RESET}"
  local install_command_display="sudo pacman -S --noconfirm ${dep_name}"
  echo -e "Install using: ${CYAN}${BOLD}${install_command_display}${RESET}"
  local confirm_install=0
  if [[ $has_gum -eq 1 ]]; then
    if gum confirm "Install ${dep_name}?"; then confirm_install=1; fi
  else
    read -rp "Install ${dep_name}? (y/N): " choice
    case "$choice" in y | Y) confirm_install=1 ;; *) confirm_install=0 ;; esac
  fi

  if [[ $confirm_install -eq 1 ]]; then
    echo -e "${BLUE}Installing ${dep_name}...${RESET}"
    if sudo pacman -S --noconfirm "${dep_name}"; then
      if command -v "$dep_name" >/dev/null 2>&1; then
        echo -e "${GREEN}${dep_name} installed.${RESET}"
        return 0
      else
        echo -e "${RED}Failed to install ${dep_name} (command still not found after successful-looking install).${RESET}" >&2
      fi
    else
      echo -e "${RED}Failed to install ${dep_name} (installer reported an error).${RESET}" >&2
    fi
    if [[ "$criticality_level" == "critical" ]]; then
      echo -e "${RED}Please install manually.${RESET}" >&2
      exit 1
    else
      echo -e "${YELLOW}Continuing without optional ${dep_name}.${RESET}" >&2
      return 1
    fi
  else
    echo -e "${YELLOW}Install of ${dep_name} declined.${RESET}" >&2
    if [[ "$criticality_level" == "critical" ]]; then
      echo -e "${RED}Cannot continue.${RESET}" >&2
      exit 1
    else
      echo -e "${YELLOW}Continuing without optional ${dep_name}.${RESET}" >&2
      return 2
    fi
  fi
  # Unreachable, but to silence warnings
  return 0
}

get_installed_vscode_version() {
  local install_path_base="$1" dir_name="$2"
  local product_json_path="${install_path_base}/${dir_name}/resources/app/product.json"
  if [ -f "$product_json_path" ]; then jq -r '.version' "$product_json_path" 2>/dev/null || echo "Parse err"; else echo "Not Installed"; fi
}

get_online_vscode_version() {
  local base_download_url="$1"
  local channel
  if [[ "$base_download_url" == *"/stable"* ]]; then
    channel="stable"
  elif [[ "$base_download_url" == *"/insider"* ]]; then
    channel="insider"
  else
    debug_echo "get_online_vscode_version: Bad URL '$base_download_url'"
    echo "N/A (bad URL)"
    return 1
  fi

  local actual_api_url="https://update.code.visualstudio.com/api/update/linux-x64/${channel}/latest"
  debug_echo "Fetching online version from: $actual_api_url"
  local version_info
  version_info=$(curl -sL --connect-timeout 5 --max-time 10 "$actual_api_url")
  if [[ -n "$version_info" ]]; then
    local product_version
    product_version=$(echo "$version_info" | jq -r '.productVersion' 2>/dev/null)
    if [[ -n "$product_version" && "$product_version" != "null" ]]; then
      echo "$product_version"
    else
      debug_echo "get_online_vscode_version: jq parse error or null version for $channel"
      echo "N/A (parse)"
    fi
  else
    debug_echo "get_online_vscode_version: Fetch error for $channel from $actual_api_url"
    echo "N/A (fetch)"
  fi
}

download_and_extract() {
  local url="$1"
  local target_dir="$2"
  local archive
  mkdir -p "$APP_DOWNLOAD_DIR"
  echo -e "${YELLOW}Downloading from $url...${RESET}"
  archive="${APP_DOWNLOAD_DIR}/vscode-$(basename "$url")-$$.tar.gz"
  debug_echo "Download archive path: $archive"
  if curl -L --connect-timeout 10 --max-time 600 "$url" -o "$archive"; then
    echo -e "${YELLOW}Extracting to $target_dir...${RESET}"
    sudo mkdir -p "$target_dir"
    if sudo tar --strip-components=1 -xzf "$archive" -C "$target_dir"; then
      rm -f "$archive"
      echo -e "${GREEN}Extracted.${RESET}"
    else
      echo -e "${RED}Extract failed: $archive.${RESET}" >&2
      rm -f "$archive"
      return 1
    fi
  else
    echo -e "${RED}Download failed: $url.${RESET}" >&2
    return 1
  fi
  return 0
}

deploy_variant() {
  local label="$1"
  local url="$2"
  local dir_name="$3"
  local symlink_name="$4"
  local target="$INSTALL_PREFIX/$dir_name"

  debug_echo "Deploying variant: Label='${label}', URL='${url}', Dir='${dir_name}', Symlink='${symlink_name}', Target='${target}'"
  if [[ -z "$dir_name" ]]; then
    debug_echo "CRITICAL ERROR in deploy_variant: dir_name is empty for label '${label}'. Aborting this deployment."
    echo -e "${RED}Error: Deployment configuration issue for ${label} (empty directory name).${RESET}" >&2
    return 1
  fi

  if [[ -d "$target" ]]; then
    echo -e "${BLUE}Updating ${label} edition at $target${RESET}"
  else
    echo -e "${BLUE}Installing ${label} edition to $target${RESET}"
  fi

  if download_and_extract "$url" "$target"; then
    echo -e "${BLUE}Linking binary to /usr/local/bin/${symlink_name}...${RESET}"
    sudo ln -sf "$target/bin/${symlink_name}" "/usr/local/bin/${symlink_name}"
    echo -e "${GREEN}${label} edition deployment complete.${RESET}"
  else
    echo -e "${RED}Failed to deploy ${label} edition.${RESET}"
  fi
}

uninstall_variant() {
  local label="$1"
  local dir_name="$2"
  local symlink_name="$3"
  local target_dir="$INSTALL_PREFIX/$dir_name"
  local target_symlink="/usr/local/bin/$symlink_name"

  debug_echo "Uninstalling variant: Label='${label}', Dir='${dir_name}', Symlink='${symlink_name}', TargetDir='${target_dir}', TargetSymlink='${target_symlink}'"
  if [[ -z "$dir_name" ]]; then
    debug_echo "CRITICAL ERROR in uninstall_variant: dir_name is empty for label '${label}'. Aborting this uninstallation."
    echo -e "${RED}Error: Uninstallation configuration issue for ${label} (empty directory name).${RESET}" >&2
    return 1
  fi

  echo -e "${YELLOW}Uninstalling VSCode ${label}...${RESET}"
  if [[ ! -d "$target_dir" && ! -L "$target_symlink" && ! -f "$target_symlink" ]]; then
    echo -e "${BLUE}${label} not installed.${RESET}"
    return 0
  fi
  local confirm_uninstall=0
  echo -e "${RED}This removes:${RESET}"
  [ -d "$target_dir" ] && echo -e "${RED}- Dir: $target_dir${RESET}"
  ([ -L "$target_symlink" ] || [ -f "$target_symlink" ]) && echo -e "${RED}- Link: $target_symlink${RESET}"
  if [[ $has_gum -eq 1 ]]; then
    if gum confirm "Uninstall ${label}?"; then confirm_uninstall=1; fi
  else
    read -rp "Uninstall ${label}? (y/N): " choice
    case "$choice" in y | Y) confirm_uninstall=1 ;; *) confirm_uninstall=0 ;; esac
  fi
  if [[ $confirm_uninstall -eq 1 ]]; then
    echo -e "${BLUE}Uninstalling ${label}...${RESET}"
    if [[ -d "$target_dir" ]]; then sudo rm -rf "$target_dir" && echo -e "${GREEN}Dir removed.${RESET}" || echo -e "${RED}Fail rm dir.${RESET}"; fi
    if [[ -L "$target_symlink" || -f "$target_symlink" ]]; then sudo rm -f "$target_symlink" && echo -e "${GREEN}Link removed.${RESET}" || echo -e "${RED}Fail rm link.${RESET}"; fi
    echo -e "${GREEN}${label} uninstalled.${RESET}"
  else echo -e "${YELLOW}Uninstall of ${label} cancelled.${RESET}"; fi
}

# --- AUR Helper Installation Functions ---
install_aur_helper() {
  local repo_url="$1"
  local clone_dir="$2"
  local tmp_dir="/tmp/${clone_dir}-$$"

  debug_echo "Installing AUR helper from: $repo_url into $tmp_dir"
  echo -e "${BLUE}Installing AUR helper from: ${YELLOW}${repo_url}${RESET}"

  if ! command -v git >/dev/null 2>&1; then
    echo -e "${RED}Error: Git is required but not installed.${RESET}" >&2
    return 1
  fi

  mkdir -p "$tmp_dir"
  if ! git clone "$repo_url" "$tmp_dir"; then
    echo -e "${RED}Error: Failed to clone repository.${RESET}" >&2
    rm -rf "$tmp_dir"
    return 1
  fi

  cd "$tmp_dir" || {
    echo -e "${RED}Error: Failed to change directory to $tmp_dir.${RESET}" >&2
    return 1
  }
  if ! makepkg -si --noconfirm; then
    echo -e "${RED}Error: Failed to build or install package.${RESET}" >&2
    cd - || true
    rm -rf "$tmp_dir"
    return 1
  fi

  cd - || true
  rm -rf "$tmp_dir"
  echo -e "${GREEN}AUR helper installed successfully.${RESET}"
  return 0
}

install_yay_helper() {
  install_aur_helper "$YAY_REPO_URL" "$YAY_CLONE_DIR"
}

install_paru_helper() {
  install_aur_helper "$PARU_REPO_URL" "$PARU_CLONE_DIR"
}

install_both_helpers() {
  echo -e "${BLUE}Installing both AUR helpers...${RESET}"
  install_yay_helper
  install_paru_helper
  echo -e "${GREEN}Both AUR helpers installed.${RESET}"
}

# --- Git Configuration Functions ---
setup_git_user() {
  local git_config_json_raw git_config_json_stripped
  local git_name git_email

  # Read the git config from the JSON file
  if [[ ! -f "$GIT_CONFIG_MENU_FILE_PATH" ]]; then
    echo -e "${RED}Error: Git configuration file not found.${RESET}" >&2
    return 1
  fi

  git_config_json_raw=$(cat "$GIT_CONFIG_MENU_FILE_PATH")
  git_config_json_stripped=$(_strip_jsonc_comments "$git_config_json_raw")

  git_name=$(_get_json_value "$git_config_json_stripped" ".defaultConfig.name")
  git_email=$(_get_json_value "$git_config_json_stripped" ".defaultConfig.email")

  if [[ -z "$git_name" || -z "$git_email" ]]; then
    echo -e "${RED}Error: Git name or email not found in configuration.${RESET}" >&2
    return 1
  fi

  echo -e "${BLUE}Setting up Git user...${RESET}"
  git config --global user.name "$git_name"
  git config --global user.email "$git_email"
  echo -e "${GREEN}Git user configured successfully:${RESET}"
  echo -e "  ${WHITE}Name : ${git_name}${RESET}"
  echo -e "  ${WHITE}Email: ${git_email}${RESET}"
}

show_git_config() {
  echo -e "${BLUE}Current Git Configuration:${RESET}"
  echo -e "${YELLOW}==========================${RESET}"

  local git_user_name git_user_email
  git_user_name=$(git config --global user.name)
  git_user_email=$(git config --global user.email)

  echo -e "${WHITE}Name : ${git_user_name:-Not set}${RESET}"
  echo -e "${WHITE}Email: ${git_user_email:-Not set}${RESET}"
  echo -e "${YELLOW}==========================${RESET}"

  # Show other relevant git configurations
  echo -e "${CYAN}Other Git Settings:${RESET}"
  git config --global --list | grep -v "^user.name=" | grep -v "^user.email=" | sort
}

# --- Wrapper functions for menu actions ---
run_uname_a() {
  uname -a
  return $?
}
run_pwd() {
  pwd
  return $?
}
run_ls_lh() {
  ls -lh
  return $?
}

deploy_vscode_stable() {
  debug_echo "Function: deploy_vscode_stable called"
  deploy_variant "$VSCODE_STABLE_LABEL" "$VSCODE_STABLE_URL" "$VSCODE_STABLE_DIR_NAME" "$VSCODE_STABLE_SYMLINK_NAME"
}
deploy_vscode_insiders() {
  debug_echo "Function: deploy_vscode_insiders called"
  deploy_variant "$VSCODE_INSIDERS_LABEL" "$VSCODE_INSIDERS_URL" "$VSCODE_INSIDERS_DIR_NAME" "$VSCODE_INSIDERS_SYMLINK_NAME"
}
deploy_vscode_both() {
  debug_echo "Function: deploy_vscode_both called"
  deploy_vscode_stable
  deploy_vscode_insiders
}
uninstall_vscode_stable() {
  debug_echo "Function: uninstall_vscode_stable called"
  uninstall_variant "$VSCODE_STABLE_LABEL" "$VSCODE_STABLE_DIR_NAME" "$VSCODE_STABLE_SYMLINK_NAME"
}
uninstall_vscode_insiders() {
  debug_echo "Function: uninstall_vscode_insiders called"
  uninstall_variant "$VSCODE_INSIDERS_LABEL" "$VSCODE_INSIDERS_DIR_NAME" "$VSCODE_INSIDERS_SYMLINK_NAME"
}
uninstall_vscode_both() {
  debug_echo "Function: uninstall_vscode_both called"
  uninstall_vscode_stable
  uninstall_vscode_insiders
}
# --- End Wrapper functions ---

_substitute_placeholders_in_command() {
  local command_template_arg="$1"
  local result="$command_template_arg"

  result="${result//\{\{VSCODE_STABLE_URL\}\}/$VSCODE_STABLE_URL}"
  result="${result//\{\{VSCODE_STABLE_DIR_NAME\}\}/$VSCODE_STABLE_DIR_NAME}"
  result="${result//\{\{VSCODE_STABLE_SYMLINK_NAME\}\}/$VSCODE_STABLE_SYMLINK_NAME}"
  result="${result//\{\{VSCODE_STABLE_LABEL\}\}/$VSCODE_STABLE_LABEL}"
  result="${result//\{\{VSCODE_INSIDERS_URL\}\}/$VSCODE_INSIDERS_URL}"
  result="${result//\{\{VSCODE_INSIDERS_DIR_NAME\}\}/$VSCODE_INSIDERS_DIR_NAME}"
  result="${result//\{\{VSCODE_INSIDERS_SYMLINK_NAME\}\}/$VSCODE_INSIDERS_SYMLINK_NAME}"
  result="${result//\{\{VSCODE_INSIDERS_LABEL\}\}/$VSCODE_INSIDERS_LABEL}"

  if [[ "$command_template_arg" != "$result" ]]; then
    debug_echo "Substituted command: $result (from: $command_template_arg)"
  fi

  if [[ "$result" == *"{{WARN_UNREPLACED"* ]]; then
    debug_echo "Note: Result contains an intentionally unreplaced placeholder: $result"
  elif [[ "$result" == *"{{*}}"* ]]; then
    debug_echo "Warning: Command might contain unreplaced placeholders: $result"
  fi
  echo "$result"
}

_select_option_fzf() {
  local header_info="$1"
  local menu_title="$2"
  shift 2
  local options_texts_array=("$@")
  local selected_text
  local header_line_count fzf_header_lines

  header_line_count=$(echo -e "$header_info" | wc -l)
  fzf_header_lines=$((header_line_count + 1))

  selected_text=$( (
    echo -e "$header_info\n"
    printf '%s\n' "${options_texts_array[@]}"
  ) |
    FZF_DEFAULT_OPTS="" SHELL=/bin/bash fzf \
      --ansi --header-lines="$fzf_header_lines" --prompt="${menu_title%% *}> " \
      --border --no-multi --cycle --no-sort --height=~70% --layout=reverse-list --no-mouse)

  echo "$selected_text"
}

_select_option_gum() {
  local header_info="$1"
  local menu_title="$2"
  shift 2
  local options_texts_array=("$@")
  local selected_text

  tput clear
  echo -e "$header_info\n"
  selected_text=$(gum choose "${options_texts_array[@]}" --header "Select Action for ${menu_title}" --height 15 || true)
  echo "$selected_text"
}

_select_option_manual() {
  local header_info="$1"
  local menu_title="$2"
  local menu_file_path_context="$3"
  local current_selected_idx="$4"
  shift 4
  local options_texts_array=("$@")

  local selected_text=""
  local key_nav key_nav2
  local new_selected_idx="$current_selected_idx"

  while true; do
    tput clear
    echo -e "$header_info\n"
    echo -e "${BLUE}┌──────────────────────────────────┐${RESET}"
    for i in "${!options_texts_array[@]}"; do
      local display_text="$((i + 1))) ${options_texts_array[i]}"
      if [[ $i -eq $new_selected_idx ]]; then
        echo -e "${BLUE}│${RESET} ${INVERT}${display_text}${RESET}"
      else
        echo -e "${BLUE}│${RESET} ${display_text}"
      fi
    done
    echo -e "${BLUE}└──────────────────────────────────┘${RESET}"

    local back_option_text="0 to go back/exit"
    if [[ "$menu_file_path_context" == "$MAIN_MENU_FILE_PATH" ]]; then
      back_option_text="0 to exit"
    fi
    echo -e "Use ↑↓, ⏎, or numbers. $back_option_text."

    IFS= read -rsn1 key_nav
    if [[ $key_nav == $'\x1b' ]]; then
      read -rsn2 -t 0.1 key_nav2 || true
      key_nav+=$key_nav2
      case $key_nav in
      $'\x1b[A') ((new_selected_idx = (new_selected_idx - 1 + ${#options_texts_array[@]}) % ${#options_texts_array[@]})) ;;
      $'\x1b[B') ((new_selected_idx = (new_selected_idx + 1) % ${#options_texts_array[@]})) ;;
      esac
    elif [[ $key_nav == "" ]]; then
      selected_text="${options_texts_array[$new_selected_idx]}"
      break
    elif [[ $key_nav == "0" ]]; then
      selected_text=""
      break
    elif [[ "$key_nav" =~ ^[1-9]$ && "$key_nav" -le "${#options_texts_array[@]}" ]]; then
      new_selected_idx=$((key_nav - 1))
      selected_text="${options_texts_array[$new_selected_idx]}"
      break
    else
      echo -e "${RED}Invalid selection.${RESET}"
      sleep 1
    fi
  done
  echo "${selected_text};${new_selected_idx}"
}

_dispatch_menu_action() {
  local selected_option_json="$1"
  local selected_text="$2"
  local menu_file_path_context="$3"
  local action_status="CONTINUE"

  local action_type
  action_type=$(_get_json_value "$selected_option_json" ".type")
  debug_echo "Dispatching action for: '$selected_text', type: '$action_type', JSON: '$selected_option_json'"

  echo -e "${GREEN}Selected: ${selected_text}${RESET}"
  loading

  case "$action_type" in
  "scriptFunction")
    local func_name
    func_name=$(_get_json_value "$selected_option_json" ".functionName")
    if [[ -z "$func_name" ]]; then
      echo -e "${RED}Error: 'functionName' not specified for scriptFunction type in menu item '$selected_text'.${RESET}" >&2
      # action_status will remain "CONTINUE", leading to "Press ENTER"
    elif declare -F "$func_name" >/dev/null; then
      debug_echo "Calling script function: $func_name"
      "$func_name" # Call the predefined script function
    else
      echo -e "${RED}Error: Unknown script function '$func_name' for menu item '$selected_text'.${RESET}" >&2
    fi
    ;;
  "submenu")
    local handler_func
    handler_func=$(_get_json_value "$selected_option_json" ".handlerFunction")
    debug_echo "Calling submenu handler: $handler_func"
    if declare -F "$handler_func" >/dev/null; then
      "$handler_func"
    else
      echo -e "${RED}Err: Submenu func '$handler_func' undef.${RESET}" >&2
    fi
    ;;
  "exit")
    debug_echo "Action type: exit. Exiting script."
    echo "Exiting."
    action_status="EXIT_SCRIPT"
    ;;
  "return")
    debug_echo "Action type: return. Returning from submenu."
    action_status="RETURN_FROM_MENU"
    ;;
  *)
    echo -e "${RED}Unknown action type: '$action_type' for '$selected_text'${RESET}" >&2
    ;;
  esac

  if [[ "$menu_file_path_context" == "$VSCODE_MENU_FILE_PATH" ]] &&
    ([[ "$action_type" == "scriptFunction" ]]); then
    local func_name_for_check
    func_name_for_check=$(_get_json_value "$selected_option_json" ".functionName")
    if [[ -n "$func_name_for_check" ]] &&
      ([[ "$func_name_for_check" == "deploy_"* ]] || [[ "$func_name_for_check" == "uninstall_"* ]]); then
      echo -e "${GREEN}VSCode operation finished. Versions will refresh on next menu display.${RESET}"
    fi
  fi
  GLOBAL_ACTION_STATUS="$action_status"
}

process_menu_from_json() {
  debug_echo "process_menu_from_json called with menu file: $1"
  local menu_file_path="$1"
  local menu_json_raw menu_json_stripped menu_title
  local menu_options_texts_array=()
  local -A menu_actions_map

  if [[ ! -f "$menu_file_path" ]]; then
    echo -e "${RED}Err: Menu definition file missing: $menu_file_path${RESET}" >&2
    return 1
  fi
  menu_json_raw=$(cat "$menu_file_path")
  menu_json_stripped=$(_strip_jsonc_comments "$menu_json_raw")

  if [[ "$DEBUG_MODE" -eq 1 ]]; then
    debug_echo "Stripped menu content for '$menu_file_path' START:"
    echo "$menu_json_stripped" >&2
    debug_echo "Stripped menu content for '$menu_file_path' END."
  fi

  local jq_test_output_menu
  if ! jq_test_output_menu=$(echo "$menu_json_stripped" | jq -e . 2>&1); then
    echo -e "${RED}Err: Invalid JSON in menu file '$menu_file_path'${RESET}" >&2
    echo -e "${RED}jq error output:\n${RESET}${jq_test_output_menu}" >&2
    return 1
  fi
  menu_title=$(_get_json_value "$menu_json_stripped" ".title" "Menu")

  local option_texts_from_jq=()
  local option_objects_from_jq=()
  mapfile -t option_texts_from_jq < <(echo "$menu_json_stripped" | jq -r '.options[].text')
  mapfile -t option_objects_from_jq < <(echo "$menu_json_stripped" | jq -c '.options[]')

  for ((i = 0; i < ${#option_texts_from_jq[@]}; i++)); do
    local text="${option_texts_from_jq[i]}"
    local option_json="${option_objects_from_jq[i]}"
    if [[ -n "$text" && -n "$option_json" ]]; then
      menu_options_texts_array+=("$text")
      menu_actions_map["$text"]="$option_json"
      debug_echo "Added menu option [$i]: '$text'"
    else
      debug_echo "Skipping option index $i due to empty text or JSON in '$menu_file_path'."
    fi
  done

  if [[ ${#menu_options_texts_array[@]} -eq 0 ]]; then
    echo -e "${YELLOW}Warn: No options parsed in '$menu_file_path'${RESET}" >&2
    return 1
  fi

  local selected_text selected_option_json header_info
  local manual_nav_selected_idx=0

  debug_echo "Menu loop starting for '$menu_title'."
  while true; do
    tput clear
    header_info=""
    if [[ "$menu_file_path" == "$VSCODE_MENU_FILE_PATH" ]]; then
      local installed_stable_ver installed_insiders_ver online_stable_ver online_insiders_ver
      set +e
      installed_stable_ver=$(get_installed_vscode_version "$INSTALL_PREFIX" "$VSCODE_STABLE_DIR_NAME")
      installed_insiders_ver=$(get_installed_vscode_version "$INSTALL_PREFIX" "$VSCODE_INSIDERS_DIR_NAME")
      online_stable_ver=$(get_online_vscode_version "$VSCODE_STABLE_URL")
      online_insiders_ver=$(get_online_vscode_version "$VSCODE_INSIDERS_URL")
      set -e
      header_info=$(
        cat <<EOF
${BOLD}${BLUE}${menu_title}${RESET}

${CYAN}Installed Versions:${RESET}
$(printf "  ${WHITE}%-10s: %s${RESET}" "$VSCODE_STABLE_LABEL" "$installed_stable_ver")
$(printf "  ${WHITE}%-10s: %s${RESET}" "$VSCODE_INSIDERS_LABEL" "$installed_insiders_ver")

${CYAN}Available Online:${RESET}
$(printf "  ${WHITE}%-10s: %s${RESET}" "$VSCODE_STABLE_LABEL" "$online_stable_ver")
$(printf "  ${WHITE}%-10s: %s${RESET}" "$VSCODE_INSIDERS_LABEL" "$online_insiders_ver")
EOF
      )
    else
      header_info="${BOLD}${BLUE}${menu_title}${RESET}"
    fi

    selected_text=""
    if [[ $has_fzf -eq 1 ]]; then
      selected_text=$(_select_option_fzf "$header_info" "$menu_title" "${menu_options_texts_array[@]}")
    elif [[ $has_gum -eq 1 ]]; then
      selected_text=$(_select_option_gum "$header_info" "$menu_title" "${menu_options_texts_array[@]}")
    else
      local manual_result
      manual_result=$(_select_option_manual "$header_info" "$menu_title" "$menu_file_path" "$manual_nav_selected_idx" "${menu_options_texts_array[@]}")
      selected_text=$(echo "$manual_result" | cut -d';' -f1)
      manual_nav_selected_idx=$(echo "$manual_result" | cut -d';' -f2)
    fi

    if [[ -z "$selected_text" ]]; then
      debug_echo "No selection made or back/exit chosen in menu '$menu_title'."
      if [[ "$menu_file_path" != "$MAIN_MENU_FILE_PATH" ]]; then
        return 0
      else
        tput clear
        echo "Exiting."
        exit 0
      fi
    fi

    selected_option_json="${menu_actions_map[$selected_text]}"
    if [[ -z "$selected_option_json" ]]; then
      echo -e "${RED}Error: Could not find action for selection '$selected_text'.${RESET}" >&2
      sleep 2
      continue
    fi

    local dispatched_action_type
    dispatched_action_type=$(_get_json_value "$selected_option_json" ".type")

    _dispatch_menu_action "$selected_option_json" "$selected_text" "$menu_file_path"

    local should_prompt_for_enter=0

    case "$GLOBAL_ACTION_STATUS" in
    "EXIT_SCRIPT")
      exit 0
      ;;
    "RETURN_FROM_MENU")
      if [[ "$menu_file_path" != "$MAIN_MENU_FILE_PATH" ]]; then
        return 0
      else
        debug_echo "Warning: Main menu encountered RETURN_FROM_MENU. Treating as loop continuation."
        # No prompt, just loop to redisplay main menu
      fi
      ;;
    "CONTINUE")
      if [[ "$dispatched_action_type" == "scriptFunction" ]]; then
        should_prompt_for_enter=1
      # If dispatched_action_type was "submenu", GLOBAL_ACTION_STATUS is "CONTINUE",
      # but we don't prompt. The loop continues and redraws the current (parent) menu.
      fi
      ;;
    *)
      debug_echo "Unexpected GLOBAL_ACTION_STATUS: '$GLOBAL_ACTION_STATUS' from action type '$dispatched_action_type'. Defaulting to prompt."
      should_prompt_for_enter=1
      ;;
    esac

    if [[ "$should_prompt_for_enter" -eq 1 ]]; then
      echo
      read -rp "Press ENTER to continue..." _
    fi
  done
  debug_echo "Menu loop finished for '$menu_title'" # Unreachable
  return 0                                          # Unreachable
}

vscode_menu_from_json() {
  debug_echo "vscode_menu_from_json called."
  process_menu_from_json "$VSCODE_MENU_FILE_PATH"
}

aur_helpers_menu_from_json() {
  debug_echo "aur_helpers_menu_from_json called."
  process_menu_from_json "$AUR_HELPERS_MENU_FILE_PATH"
}

git_config_menu_from_json() {
  debug_echo "git_config_menu_from_json called."
  process_menu_from_json "$GIT_CONFIG_MENU_FILE_PATH"
}

loading() {
  if [[ $has_gum -eq 1 ]]; then gum spin --title "Loading..." -- sleep "$anim_delay_sec"; else
    echo -n "Loading"
    for ((i = 0; i < ANIM_STEPS; i++)); do
      sleep "$anim_step_size"
      echo -n "."
    done
    echo
  fi
}

# --- Main Script Execution ---
debug_echo "Setting strict mode."
set -euo pipefail
debug_echo "Strict mode set."

debug_echo "Calling parse_cli_args (final pass for direct actions)."
parse_cli_args "$@"
debug_echo "parse_cli_args (final pass) returned."

debug_echo "Checking TUI dependencies."
default_check_dep bc "critical"
anim_delay_sec=$(echo "scale=3; $ANIM_DELAY_MS / 1000" | bc)
anim_step_size=$(echo "scale=4; $ANIM_DELAY_MS / 1000 / $ANIM_STEPS" | bc)
debug_echo "Animation variables calculated: sec=$anim_delay_sec, step=$anim_step_size"

default_check_dep curl "critical"
default_check_dep tar "critical"
debug_echo "Critical TUI dependencies (bc, curl, tar) checked."

default_check_dep fzf "optional" || true
default_check_dep gum "optional" || true
has_fzf=$(command -v fzf &>/dev/null && echo 1 || echo 0)
has_gum=$(command -v gum &>/dev/null && echo 1 || echo 0)
debug_echo "Optional tools checked/re-checked: fzf=$has_fzf, gum=$has_gum"

debug_echo "Starting Main Menu from MAIN_MENU_FILE_PATH: '$MAIN_MENU_FILE_PATH'"
if [[ -z "$MAIN_MENU_FILE_PATH" ]]; then
  echo -e "${RED}FATAL: MAIN_MENU_FILE_PATH is not set. Cannot start main menu.${RESET}" >&2
  exit 1
elif [[ ! -f "$MAIN_MENU_FILE_PATH" ]]; then
  echo -e "${RED}FATAL: Main menu file '$MAIN_MENU_FILE_PATH' does not exist. Cannot start main menu.${RESET}" >&2
  exit 1
fi

process_menu_from_json "$MAIN_MENU_FILE_PATH"

debug_echo "process_menu_from_json (main menu) returned. Script ending."
echo "Main menu process finished."
exit 0
