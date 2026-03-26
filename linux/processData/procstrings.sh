#!/usr/bin/env bash
#
# procstrings.sh — Extract printable strings from a running process's memory
#
# This script reads /proc/<pid>/maps and /proc/<pid>/mem directly.
# It does NOT use ptrace, send signals, or attach to the process in any way,
# so the target process is completely unaffected.
#
# Requirements:
#   - Linux with /proc filesystem
#   - Read access to /proc/<pid>/mem (same user or root)
#   - dd, grep, awk (standard utilities)
#
# Usage: ./procstrings.sh [--min-len N] [--pid PID]
#

set -euo pipefail

# ─── Defaults ────────────────────────────────────────────────────────────────
MIN_STRING_LEN=4
TARGET_PID=""

# ─── Parse arguments ─────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --min-len)
            MIN_STRING_LEN="$2"
            shift 2
            ;;
        --pid)
            TARGET_PID="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [--min-len N] [--pid PID]"
            echo ""
            echo "  --min-len N   Minimum string length to extract (default: 4)"
            echo "  --pid PID     Skip interactive menu and use this PID directly"
            echo ""
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

# ─── Colours (disabled if not a terminal) ─────────────────────────────────────
if [[ -t 1 ]]; then
    BOLD=$'\033[1m'
    DIM=$'\033[2m'
    CYAN=$'\033[36m'
    GREEN=$'\033[32m'
    YELLOW=$'\033[33m'
    RED=$'\033[31m'
    RESET=$'\033[0m'
else
    BOLD="" DIM="" CYAN="" GREEN="" YELLOW="" RED="" RESET=""
fi

# ─── Helper: print styled header ─────────────────────────────────────────────
header() {
    echo ""
    echo "${BOLD}${CYAN}══════════════════════════════════════════════════════════════${RESET}"
    echo "${BOLD}${CYAN}  $1${RESET}"
    echo "${BOLD}${CYAN}══════════════════════════════════════════════════════════════${RESET}"
    echo ""
}

# ─── Helper: error + exit ────────────────────────────────────────────────────
die() {
    echo "${RED}[ERROR]${RESET} $*" >&2
    exit 1
}

# ─── Validate environment ────────────────────────────────────────────────────
[[ -d /proc ]] || die "/proc filesystem not found. This script requires Linux."

# ─── Step 1: List processes ──────────────────────────────────────────────────
if [[ -z "$TARGET_PID" ]]; then
    header "Running processes for user: $(whoami)"

    printf "${BOLD}%-8s %-6s %-6s %-20s %s${RESET}\n" "PID" "%CPU" "%MEM" "STARTED" "COMMAND"
    echo "${DIM}──────── ────── ────── ──────────────────── ──────────────────────${RESET}"

    ps -u "$(whoami)" -o pid=,pcpu=,pmem=,lstart=,comm= --sort=-%mem \
        | while read -r pid cpu mem d1 d2 d3 d4 d5 comm; do
            started="$d1 $d2 $d3 $d4 $d5"
            printf "%-8s %-6s %-6s %-20s %s\n" "$pid" "$cpu" "$mem" "$started" "$comm"
        done

    echo ""
    echo -n "${GREEN}Enter PID to extract strings from (or 'q' to quit): ${RESET}"
    read -r TARGET_PID

    [[ "$TARGET_PID" == "q" || "$TARGET_PID" == "Q" ]] && { echo "Bye."; exit 0; }
fi

# ─── Step 2: Validate PID ────────────────────────────────────────────────────
# Must be a number
[[ "$TARGET_PID" =~ ^[0-9]+$ ]] || die "'$TARGET_PID' is not a valid PID."

# Must exist
[[ -d "/proc/$TARGET_PID" ]] || die "PID $TARGET_PID does not exist."

# Must have readable maps
MAPS_FILE="/proc/$TARGET_PID/maps"
MEM_FILE="/proc/$TARGET_PID/mem"

[[ -r "$MAPS_FILE" ]] || die "Cannot read $MAPS_FILE — do you own this process or have root?"
[[ -r "$MEM_FILE"  ]] || die "Cannot read $MEM_FILE — do you own this process or have root?"

PROC_NAME=$(cat "/proc/$TARGET_PID/comm" 2>/dev/null || echo "unknown")
header "Extracting strings from PID $TARGET_PID ($PROC_NAME)"
echo "${DIM}Minimum string length: $MIN_STRING_LEN${RESET}"
echo "${DIM}Reading memory regions from $MAPS_FILE …${RESET}"
echo ""

# ─── Step 3: Read memory and extract strings ─────────────────────────────────
#
# Strategy:
#   1. Parse /proc/<pid>/maps for readable regions (lines with 'r' permission).
#   2. For each region, use dd to read the bytes from /proc/<pid>/mem.
#   3. Pipe through `strings` (or grep-based equivalent) to find printable sequences.
#
# This is a PASSIVE read — the kernel serves the data from the process's page
# tables without stopping, signalling, or otherwise affecting the process.
#

REGION_COUNT=0
STRING_COUNT=0

while IFS= read -r line; do
    # Example maps line:
    # 5574a0c00000-5574a0c20000 r--p 00000000 08:01 1234  /usr/bin/foo
    addr_range=$(echo "$line" | awk '{print $1}')
    perms=$(echo "$line" | awk '{print $2}')

    # Only read regions that are readable
    [[ "${perms:0:1}" == "r" ]] || continue

    start_hex="${addr_range%-*}"
    end_hex="${addr_range#*-}"

    # Convert hex to decimal
    start_dec=$((16#$start_hex))
    end_dec=$((16#$end_hex))
    size=$((end_dec - start_dec))

    # Skip empty or absurdly large regions (> 256 MB) to avoid hanging
    [[ $size -le 0 ]] && continue
    [[ $size -gt $((256 * 1024 * 1024)) ]] && continue

    REGION_COUNT=$((REGION_COUNT + 1))

    # Read the region and extract strings
    # dd reads from mem at the right offset; errors (e.g. unmapped pages) are
    # silently discarded via 2>/dev/null. The process is never touched.
    dd if="$MEM_FILE" bs=1 skip="$start_dec" count="$size" 2>/dev/null \
        | strings -n "$MIN_STRING_LEN" 2>/dev/null \
        && true   # don't abort on dd read errors (EFAULT on guard pages, etc.)

done < <(cat "$MAPS_FILE" 2>/dev/null)

echo "" >&2
echo "${GREEN}Done.${RESET} Scanned ${BOLD}$REGION_COUNT${RESET} readable memory regions from PID $TARGET_PID." >&2
