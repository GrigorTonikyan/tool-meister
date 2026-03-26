#!/bin/bash
#
# show_env.sh - Displays environment variables in a formatted and colorful way.
#

# --- Color Definitions ---
C_HEADER='\033[1;35m' # Bold Magenta
C_GROUP='\033[1;34m'  # Bold Blue
C_KEY='\033[0;32m'    # Green
C_VALUE='\033[0;33m'  # Yellow
C_RESET='\033[0m'     # No Color

# --- Helper Functions ---

# Prints a formatted group header
print_header() {
    local title=$1
    printf "\n${C_HEADER}--- %-60s ---${C_RESET}\n" "$title"
}

# --- Main Logic ---

# Use an associative array to hold variables for each group
declare -A groups

# Read all environment variables and their values
# 'export -p' is used for a reliable format: declare -x VAR="VALUE"
while IFS= read -r line; do
    # Extract VAR="VALUE" part
    if [[ $line =~ ^declare\ -x\ ([^=]+)=(.*) ]]; then
        key="${BASH_REMATCH[1]}"
        value="${BASH_REMATCH[2]}"

        # Remove quotes from value if present
        if [[ $value =~ ^\"(.*)\"$ || $value =~ ^\'(.*)\' ]]; then
            value="${BASH_REMATCH[1]}"
        fi

        # Assign key to a group
        case $key in
            USER|LOGNAME|HOME|SHELL|EDITOR|TERM)
                group="User & Shell"
                ;;
            PATH|LD_LIBRARY_PATH|MANPATH|INFOPATH)
                group="Paths & Libraries"
                ;;
            DISPLAY|WAYLAND_DISPLAY|XDG_*|DBUS_*)
                group="Session & Desktop"
                ;;
            GTK_*|QT_*|THEME|XCURSOR_*)
                group="GUI & Theming"
                ;;
            LANG|LC_*|LANGUAGE)
                group="Locale & Language"
                ;;
            SSH_*|GPG_*)
                group="Security & Agents"
                ;;
            *)
                group="Miscellaneous"
                ;;
        esac

        # Store the formatted string in the associative array, keyed by group
        groups["$group"]+="${C_KEY}${key}=${C_VALUE}${value}${C_RESET}\n"
    fi
done < <(export -p)

# --- Output ---

# Get sorted group names
IFS=$'\n' sorted_groups=($(sort <<<"${!groups[*]}"))
unset IFS

# Print each group and its variables
for group in "${sorted_groups[@]}"; do
    print_header "$group"
    # Sort variables within the group alphabetically and print
    printf "%b" "${groups[$group]}" | sort
done

printf "\n"
