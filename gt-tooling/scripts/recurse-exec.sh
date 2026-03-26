#!/bin/bash

# Define the command to run for each folder containing package.json
COMMAND="pnpm install --ignore-workspace"

# Set up colors for better output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Log file for script execution
LOG_FILE="$(date +%Y%m%d-%H%M%S)-script_execution.log"

# Function to display menu options
function show_menu() {
  clear
  echo -e "${BLUE}Script Menu Options:${NC}"
  echo "1. Run the script"
  echo "2. Change command to run (current: ${YELLOW}${COMMAND}${NC})"
  echo "3. Exit"
  echo -n "Enter your choice: "
}

# Function to handle user input
function handle_input() {
  read choice
  case $choice in
  1)
    run_script
    ;;
  2)
    echo -n "Enter new command to run: "
    read new_command
    COMMAND="$new_command"
    export COMMAND
    echo -e "${GREEN}Command updated successfully!${NC}"
    ;;
  3)
    echo -e "${BLUE}Exiting...${NC}"
    exit 0
    ;;
  *)
    echo -e "${RED}Invalid choice, please try again.${NC}"
    ;;
  esac
}

# Function to run the main script
function run_script() {
  # Ensure the script is started from within a tmux session
  if [ -z "$TMUX" ]; then
    echo -e "${YELLOW}Starting a new tmux session...${NC}"
    tmux new-session -d -s "script_session" "$0 --tmux-started"
    tmux attach -t "script_session"
    exit 0
  fi

  # Check for necessary tools and install if not present
  REQUIRED_TOOLS=(tmux parallel)
  for tool in "${REQUIRED_TOOLS[@]}"; do
    if ! command -v $tool &>/dev/null; then
      echo -e "${YELLOW}Installing $tool...${NC}"
      if [ "$(uname)" == "Darwin" ]; then
        brew install $tool
      elif [ -f "/etc/debian_version" ]; then
        sudo apt-get install -y $tool
      elif [ -f "/etc/redhat-release" ]; then
        sudo yum install -y $tool
      else
        echo -e "${RED}Unsupported OS for automatic installation.${NC} Please install $tool manually."
        exit 1
      fi
    fi
  done

  # Find all package.json files excluding node_modules directories
  echo -e "${BLUE}Discovering package.json files...${NC}"
  echo "Discovering package.json files..." >>"$LOG_FILE"
  package_dirs=$(find . -name "package.json" -not -path "*/node_modules/*" -print0 | xargs -0 -I {} dirname "{}")

  # Define the total number of tasks
  TOTAL_TASKS=$(echo "$package_dirs" | wc -l)

  # Ensure that the number of tasks is not zero
  if [ $TOTAL_TASKS -eq 0 ]; then
    echo -e "${RED}No package.json files found. Exiting...${NC}"
    echo "No package.json files found. Exiting..." >>"$LOG_FILE"
    return
  fi

  # List discovered projects and provide an estimation
  echo -e "${BLUE}Discovered projects:${NC}"
  echo "$package_dirs" | while read dir; do
    echo -e "  - ${YELLOW}$dir${NC}"
    echo "  - $dir" >>"$LOG_FILE"
  done
  estimated_time=$((TOTAL_TASKS * 5)) # Assuming each task takes about 5 seconds
  echo -e "${YELLOW}Estimated time to complete: ${estimated_time} seconds${NC}"
  echo "Estimated time to complete: ${estimated_time} seconds" >>"$LOG_FILE"

  # Prompt user to continue
  echo -n "Do you want to proceed? (y/n): "
  read proceed
  if [[ "$proceed" != "y" ]]; then
    echo -e "${RED}Operation cancelled by user.${NC}"
    echo "Operation cancelled by user." >>"$LOG_FILE"
    return
  fi

  # Run commands in parallel using GNU parallel, limiting to the number of CPU cores
  CPU_CORES=$(nproc)

  echo "$package_dirs" | while read dir; do
    (
      echo -e "${YELLOW}Running command in: $dir${NC}"
      echo "Running command in: $dir" >>"$LOG_FILE"
      cd "$dir" && {
        eval "$COMMAND" 2>&1 | tee -a "$LOG_FILE" | while IFS= read -r line; do
          echo "[$dir] $line" >>"$LOG_FILE"
        done
        if [ ${PIPESTATUS[0]} -eq 0 ]; then
          echo -e "${GREEN}Success: $dir${NC}"
          echo "Success: $dir" >>"$LOG_FILE"
        else
          echo -e "${RED}Failed: $dir - Check log for details${NC}"
          echo "Failed: $dir - Check log for details" >>"$LOG_FILE"
        fi
      }
    ) &
  done

  # Wait for all background processes to complete
  wait

  # Keep the output on screen for review
  echo -e "${BLUE}All tasks completed!${NC}"
  echo "All tasks completed!" >>"$LOG_FILE"
  echo -e "${YELLOW}Review the log file: $LOG_FILE for details.${NC}"

  # Pause to allow user to review output
  read -p "Press Enter to continue..."
}

# Main loop to display menu and handle input
if [[ "$1" == "--tmux-started" ]]; then
  while true; do
    show_menu
    handle_input
  done
else
  run_script
fi
