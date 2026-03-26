#!/bin/bash

# ==========================================
# W3C MULTI-PANE TMUX DASHBOARD (V3)
# ==========================================

if [ "$EUID" -ne 0 ]; then
    echo "Please run this script as root: sudo ./w3c_awake.sh --monitor"
    exit 1
fi

NICE_LEVEL="-2"

# --- HELPER FUNCTIONS ---
get_descendants() {
    local parent=$1
    local children=$(pgrep -P "$parent" 2>/dev/null)
    for pid in $children; do
        echo "$pid"
        get_descendants "$pid"
    done
}

get_all_pids() {
    local base_pids=$(pgrep -fi "W3Champions|Warcraft|flo|Agent.exe|wineserver" 2>/dev/null)
    local all=""
    for pid in $base_pids; do
        all="$all $pid $(get_descendants "$pid")"
    done
    echo "$all" | tr ' ' '\n' | sort -u | tr '\n' ' ' | xargs
}

# --- MODE: ORCHESTRATOR (Sets up Tmux & Watch) ---
if [[ "$1" == "--monitor" ]]; then
    if ! command -v tmux &> /dev/null || ! command -v watch &> /dev/null; then
        echo "Error: 'tmux' and 'watch' are required."
        exit 1
    fi

    # Kill any old buggy dashboard sessions
    tmux kill-session -t w3c_dash 2>/dev/null

    # Create new session: Left pane uses 'watch' to redraw cleanly every 2 seconds
    tmux new-session -d -s w3c_dash "watch --color -t -n 2 '$0 --ui-proc'"
    
    # Split window: Right pane uses 'watch' to redraw cleanly every 2 seconds
    tmux split-window -h -p 50 "watch --color -t -n 2 '$0 --ui-net'"
    
    # Enable mouse mode for pane clicking/resizing
    tmux set-option -g mouse on
    
    # Attach to the clean dashboard
    tmux attach-session -t w3c_dash
    exit 0
fi

# --- MODE: LEFT COLUMN (Process Tree) ---
if [[ "$1" == "--ui-proc" ]]; then
    ALL_PIDS=$(get_all_pids)
    
    echo -e "\033[1;37m$(date '+%H:%M:%S') - W3C PROCESS TREE\033[0m"
    echo -e "=================================================="
    
    if [ -n "$ALL_PIDS" ]; then
        GAME_PIDS=""
        NET_PIDS=""
        UI_PIDS=""
        WINE_PIDS=""
        
        for pid in $ALL_PIDS; do
            CMD=$(ps -o args= -p "$pid" 2>/dev/null)
            if echo "$CMD" | grep -iqE "Warcraft|war3\.exe"; then
                GAME_PIDS+="$pid "; renice -n "$NICE_LEVEL" -p "$pid" > /dev/null 2>&1
            elif echo "$CMD" | grep -iqE "flo-worker|flo|Agent\.exe|mDNSResponder"; then
                NET_PIDS+="$pid "; renice -n "$NICE_LEVEL" -p "$pid" > /dev/null 2>&1
            elif echo "$CMD" | grep -iqE "W3Champions|CrBrowser|CrRenderer|CrGpu|CrUtility|webview"; then
                UI_PIDS+="$pid "; renice -n "$NICE_LEVEL" -p "$pid" > /dev/null 2>&1
            elif echo "$CMD" | grep -iqE "wineserver"; then
                WINE_PIDS+="$pid "; renice -n "$NICE_LEVEL" -p "$pid" > /dev/null 2>&1
            else
                renice -n 0 -p "$pid" > /dev/null 2>&1
            fi
        done

        echo -e "PID      STAT  NI  COMMAND"
        echo -e "--------------------------------------------------"
        
        TREE_DATA=$(ps -o pid=,stat=,ni=,comm= --forest -p $ALL_PIDS 2>/dev/null | sed 's/\\_/└─/g; s/|/│/g; s/+-/├─/g')
        echo "$TREE_DATA" | awk -v g="$GAME_PIDS" -v n="$NET_PIDS" -v u="$UI_PIDS" -v w="$WINE_PIDS" '
            BEGIN {
                split(g, ga, " "); for(i in ga) GAME[ga[i]]=1;
                split(n, na, " "); for(i in na) NET[na[i]]=1;
                split(u, ua, " "); for(i in ua) UI[ua[i]]=1;
                split(w, wa, " "); for(i in wa) WINE[wa[i]]=1;
            }
            {
                if ($1 == "") next;
                if ($1 in GAME) { print "\033[1;35m" $0 "\033[0m" }
                else if ($1 in NET) { print "\033[1;36m" $0 "\033[0m" }
                else if ($1 in UI) { print "\033[1;33m" $0 "\033[0m" }
                else if ($1 in WINE) { print "\033[1;32m" $0 "\033[0m" }
                else { print "\033[2;37m" $0 "\033[0m" }
            }
        '
        echo -e "\n\033[1;35m[Red]: Game\033[0m | \033[1;36m[Cyan]: Network\033[0m | \033[1;33m[Yellow]: W3C UI\033[0m"
        echo -e "\033[1;32m[Green]: Wine\033[0m | \033[2;37m[Dim]: Ignored/Reset\033[0m"
    else
        echo "Waiting for processes to launch..."
    fi
    exit 0
fi

# --- MODE: RIGHT COLUMN (Network Sockets) ---
if [[ "$1" == "--ui-net" ]]; then
    ALL_PIDS=$(get_all_pids)
    echo -e "\033[1;37m$(date '+%H:%M:%S') - W3C NETWORK SOCKETS\033[0m"
    echo -e "=========================================================================="
    
    if [ -n "$ALL_PIDS" ]; then
        PID_REGEX=$(echo "$ALL_PIDS" | tr ' ' '|')
        NET_DATA=$(ss -tunap 2>/dev/null | grep -E "pid=($PID_REGEX),")
        
        if [ -n "$NET_DATA" ]; then
            echo "$NET_DATA" | awk '
                BEGIN {
                    printf "%-5s %-25s %-25s %s\n", "PROT", "LOCAL_IP:PORT", "PEER_IP:PORT", "PROCESS"
                    print "--------------------------------------------------------------------------"
                }
                {
                    proto = toupper($1)
                    local_addr = $5
                    peer_addr = $6
                    match($0, /users:\(\("([^"]+)"/, arr)
                    proc = arr[1] ? arr[1] : "Unknown"
                    
                    color = "\033[2;37m"
                    if (proc ~ /Warcraft|war3/) color = "\033[1;35m"
                    else if (proc ~ /flo|Agent|mDNS/) color = "\033[1;36m"
                    # FIXED: Added the missing Chromium UI threads to the yellow filter
                    else if (proc ~ /W3Champions|CrBrowser|CrRenderer|CrGpu|CrUtility|webview/) color = "\033[1;33m"
                    else if (proc ~ /wineserver/) color = "\033[1;32m"
                    
                    printf "%s%-5s %-25s %-25s %s\033[0m\n", color, proto, local_addr, peer_addr, proc
                }
            '
        else
            echo "No active connections linked to W3C/Warcraft yet."
        fi
    else
         echo "Waiting for processes to launch..."
    fi
    exit 0
fi

