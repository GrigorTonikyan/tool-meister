#!/bin/bash

# ==========================================
# CONFIGURATION
# ==========================================
TARGET_PROCESS_NAME="W3Champions"
EXPECTED_SIBLINGS=("webview" "wineserver" "Warcraft") 

# ONLY processes containing these strings will get prioritized.
# This prevents crashing the 'Flo' network daemon.
WHITELIST=("Warcraft" "wineserver" "W3Champions" "msedgewebview2")

NICE_LEVEL="-5"
IONICE_CLASS="2"
IONICE_LEVEL="0"

# ==========================================
# ARGUMENT PARSING
# ==========================================
MONITOR_MODE=false
for arg in "$@"; do
    if [[ "$arg" == "--monitor" ]]; then
        MONITOR_MODE=true
    fi
done

# ==========================================
# FUNCTIONS
# ==========================================

get_descendants() {
    local parent=$1
    local children=$(pgrep -P "$parent")
    for pid in $children; do
        echo "$pid"
        get_descendants "$pid"
    done
}

find_target_parent() {
    local target_pids=$(pgrep -f "$TARGET_PROCESS_NAME")
    if [ -z "$target_pids" ]; then
        return 1
    fi

    for pid in $target_pids; do
        local ppid=$(ps -o ppid= -p "$pid" | tr -d ' ')
        
        if [ -z "$ppid" ] || [ "$ppid" -eq 1 ]; then
            continue
        fi

        local descendants=$(get_descendants "$ppid")
        if [ -n "$descendants" ]; then
            local descendant_cmdlines=$(ps -o args= -p $descendants)
            for expected in "${EXPECTED_SIBLINGS[@]}"; do
                if echo "$descendant_cmdlines" | grep -iq "$expected"; then
                    echo "$ppid"
                    return 0
                fi
            done
        fi
    done
    return 1
}

apply_priorities() {
    local pids=$1
    for pid in $pids; do
        # Get the short command name for this PID
        local comm_name=$(ps -o comm= -p "$pid")
        
        # Check if this process matches our whitelist
        local safe_to_boost=false
        for safe_proc in "${WHITELIST[@]}"; do
            if echo "$comm_name" | grep -iq "$safe_proc"; then
                safe_to_boost=true
                break
            fi
        done

        if [ "$safe_to_boost" = true ]; then
            local current_nice=$(ps -o ni= -p "$pid" | tr -d ' ')
            if [ "$current_nice" != "$NICE_LEVEL" ]; then
                sudo renice -n "$NICE_LEVEL" -p "$pid" > /dev/null 2>&1
                sudo ionice -c "$IONICE_CLASS" -n "$IONICE_LEVEL" -p "$pid" > /dev/null 2>&1
            fi
        fi
    done
}

# ==========================================
# MAIN EXECUTION
# ==========================================

if [ "$MONITOR_MODE" = true ]; then
    # Clear screen once initially
    printf "\033[2J" 
    
    while true; do
        PARENT_PID=$(find_target_parent)
        
        # Output building starts here to prevent flicker
        OUTPUT="\033[1;36m$(date '+%H:%M:%S') - W3Champions Active Monitor\033[0m\n"
        OUTPUT+="=======================================================================\n"
        
        if [ -n "$PARENT_PID" ]; then
            ALL_PIDS="$PARENT_PID $(get_descendants "$PARENT_PID")"
            apply_priorities "$ALL_PIDS"
            
            OUTPUT+="PID      STAT  NI  %CPU  COMMAND\n"
            OUTPUT+="-----------------------------------------------------------------------\n"
            
            # Generate the tree. Using 'comm=' to hide the massive arguments.
            # Awk is used to colorize prioritized processes green.
            TREE_DATA=$(ps -o pid=,stat=,ni=,pcpu=,comm= --forest -p $ALL_PIDS | sed 's/\\_/└─/g; s/|/│/g; s/+-/├─/g')
            
            FORMATTED_TREE=$(echo "$TREE_DATA" | awk -v nice="$NICE_LEVEL" '{
                if ($3 == nice) { 
                    # Prioritized processes get colored green
                    printf "\033[1;32m%-8s %-5s %-3s %-5s %s\033[0m\n", $1, $2, $3, $4, substr($0, index($0,$5)) 
                } else { 
                    # Standard processes remain default color
                    printf "%-8s %-5s %-3s %-5s %s\n", $1, $2, $3, $4, substr($0, index($0,$5)) 
                }
            }')
            
            OUTPUT+="$FORMATTED_TREE\n"
            OUTPUT+="\n\033[1;33m[!]\033[0m Green items are whitelist-prioritized.\n"
        else
            OUTPUT+="Waiting for $TARGET_PROCESS_NAME to launch...\n"
        fi
        
        # Move cursor to top-left, print everything, then clear any leftover lines below
        printf "\033[H"
        echo -e "$OUTPUT"
        printf "\033[J"
        
        sleep 2
    done
else
    # Single execution mode
    PARENT_PID=$(find_target_parent)
    if [ -z "$PARENT_PID" ]; then
        echo "Error: Could not find active process tree."
        exit 1
    fi
    ALL_PIDS="$PARENT_PID $(get_descendants "$PARENT_PID")"
    apply_priorities "$ALL_PIDS"
    echo "Done. Whitelisted processes have been prioritized safely."
fi
