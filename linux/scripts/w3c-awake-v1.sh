#!/bin/bash

# ==========================================
# CONFIGURATION
# ==========================================
TARGET_PROCESS_NAME="W3Champions"
# Added "Warcraft" so it validates the tree even after the game launches
EXPECTED_SIBLINGS=("webview" "wineserver" "Warcraft") 

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
        # Only renice if it isn't already at the target nice level to save CPU cycles
        local current_nice=$(ps -o ni= -p "$pid" | tr -d ' ')
        if [ "$current_nice" != "$NICE_LEVEL" ]; then
            sudo renice -n "$NICE_LEVEL" -p "$pid" > /dev/null 2>&1
            sudo ionice -c "$IONICE_CLASS" -n "$IONICE_LEVEL" -p "$pid" > /dev/null 2>&1
        fi
    done
}

# ==========================================
# MAIN EXECUTION
# ==========================================

if [ "$MONITOR_MODE" = true ]; then
    echo "Starting W3C Monitor Mode... (Press CTRL+C to exit)"
    sleep 1
    
    while true; do
        PARENT_PID=$(find_target_parent)
        
        clear
        date "+%H:%M:%S - Monitoring W3Champions Tree"
        echo "======================================================================="
        
        if [ -n "$PARENT_PID" ]; then
            ALL_PIDS="$PARENT_PID $(get_descendants "$PARENT_PID")"
            apply_priorities "$ALL_PIDS"
            
            # Display process tree with PID, Status, Nice Value, CPU%, and Command Name
            echo "PID      STAT  NI  %CPU  COMMAND"
            echo "-----------------------------------------------------------------------"
            ps -o pid=,stat=,ni=,pcpu=,args= --forest -p $ALL_PIDS | sed 's/\\_/└─/g; s/|/│/g; s/+-/├─/g' | awk '{printf "%-8s %-5s %-3s %-5s %s\n", $1, $2, $3, $4, substr($0, index($0,$5))}'
            
            echo -e "\n[!] Status Key: S=Sleeping, R=Running, <=High Priority, l=Multi-threaded"
        else
            echo "Waiting for $TARGET_PROCESS_NAME to launch..."
        fi
        
        sleep 2
    done
else
    # Single execution mode (no --monitor)
    echo "Scanning for target process: $TARGET_PROCESS_NAME..."
    PARENT_PID=$(find_target_parent)

    if [ -z "$PARENT_PID" ]; then
        echo "Error: Could not find active process tree for '$TARGET_PROCESS_NAME'."
        exit 1
    fi

    echo "Success: Verified parent bwrap PID is $PARENT_PID."
    ALL_PIDS="$PARENT_PID $(get_descendants "$PARENT_PID")"
    
    apply_priorities "$ALL_PIDS"
    
    # Count how many processes were affected
    COUNT=$(echo "$ALL_PIDS" | wc -w)
    echo "Done. Prevented sleep states by prioritizing $COUNT processes."
fi
