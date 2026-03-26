#!/bin/bash

# ==========================================
# CONFIGURATION
# ==========================================
TARGET_PROCESS_NAME="W3Champions"
EXPECTED_SIBLINGS=("webview" "wineserver" "Warcraft") 

# Added 'flo' to protect the networking daemon from being starved!
WHITELIST=("Warcraft" "wineserver" "W3Champions" "msedgewebview2" "flo")

# Softened to -2. Aggressive values like -5 starve network sockets during heavy 3D rendering.
NICE_LEVEL="-2"      
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

# Global variable to pass boosted PIDs to the UI formatter
BOOSTED_PIDS_LIST=" " 

apply_priorities() {
    local pids=$1
    BOOSTED_PIDS_LIST=" "
    
    for pid in $pids; do
        local comm_name=$(ps -o comm= -p "$pid")
        local is_whitelisted=false
        
        for safe_proc in "${WHITELIST[@]}"; do
            if echo "$comm_name" | grep -iq "$safe_proc"; then
                is_whitelisted=true
                break
            fi
        done

        if [ "$is_whitelisted" = true ]; then
            BOOSTED_PIDS_LIST+="${pid} "
            local current_nice=$(ps -o ni= -p "$pid" | tr -d ' ')
            if [ "$current_nice" != "$NICE_LEVEL" ]; then
                sudo renice -n "$NICE_LEVEL" -p "$pid" > /dev/null 2>&1
                sudo ionice -c "$IONICE_CLASS" -n "$IONICE_LEVEL" -p "$pid" > /dev/null 2>&1
            fi
        else
            # FORCE RESET: Reverts inherited priorities on background clutter (explorer.exe, tabtip, etc.)
            local current_nice=$(ps -o ni= -p "$pid" | tr -d ' ')
            if [ "$current_nice" != "0" ]; then
                sudo renice -n 0 -p "$pid" > /dev/null 2>&1
                sudo ionice -c 2 -n 4 -p "$pid" > /dev/null 2>&1
            fi
        fi
    done
}

# ==========================================
# MAIN EXECUTION
# ==========================================

if [ "$MONITOR_MODE" = true ]; then
    printf "\033[2J" 
    
    while true; do
        PARENT_PID=$(find_target_parent)
        
        OUTPUT="\033[1;36m$(date '+%H:%M:%S') - W3Champions Active Monitor\033[0m\n"
        OUTPUT+="=======================================================================\n"
        
        if [ -n "$PARENT_PID" ]; then
            ALL_PIDS="$PARENT_PID $(get_descendants "$PARENT_PID")"
            apply_priorities "$ALL_PIDS"
            
            TREE_DATA=$(ps -o pid=,stat=,ni=,pcpu=,comm= --forest -p $ALL_PIDS | sed 's/\\_/└─/g; s/|/│/g; s/+-/├─/g')
            
            # Format output: Header is Cyan, Boosted is Green, Reset/Ignored is Dim White
            FORMATTED_TREE=$(echo "$TREE_DATA" | awk -v boosted="$BOOSTED_PIDS_LIST" '{
                if ($1 == "") next;
                if ($1 == "PID") {
                    printf "\033[1;36m%-8s %-5s %-3s %-5s %s\033[0m\n", $1, $2, $3, $4, substr($0, index($0,$5))
                } else if ( index(boosted, " " $1 " ") ) { 
                    printf "\033[1;32m%-8s %-5s %-3s %-5s %s\033[0m\n", $1, $2, $3, $4, substr($0, index($0,$5)) 
                } else { 
                    printf "\033[2;37m%-8s %-5s %-3s %-5s %s\033[0m\n", $1, $2, $3, $4, substr($0, index($0,$5)) 
                }
            }')
            
            OUTPUT+="$FORMATTED_TREE\n"
            OUTPUT+="\n\033[1;32m[Green]\033[0m Actively Boosted (NI $NICE_LEVEL)  |  \033[2;37m[Dim]\033[0m Reset to Normal (NI 0)\n"
        else
            OUTPUT+="Waiting for $TARGET_PROCESS_NAME to launch...\n"
        fi
        
        printf "\033[H"
        echo -e "$OUTPUT"
        printf "\033[J"
        
        sleep 2
    done
else
    PARENT_PID=$(find_target_parent)
    if [ -z "$PARENT_PID" ]; then
        echo "Error: Could not find active process tree."
        exit 1
    fi
    ALL_PIDS="$PARENT_PID $(get_descendants "$PARENT_PID")"
    apply_priorities "$ALL_PIDS"
    echo "Done."
fi
