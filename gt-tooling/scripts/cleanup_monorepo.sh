#!/bin/bash

# Colors for output
green='\033[0;32m'
red='\033[0;31m'
yellow='\033[1;33m'
blue='\033[1;34m'
cyan='\033[1;36m'
magenta='\033[1;35m'
white='\033[1;37m'
nc='\033[0m' # No Color

# Install required tools
if ! command -v parallel &>/dev/null; then
  echo -e "${red}GNU parallel is required but not installed. Installing it now...${nc}"
  sudo apt-get install -y parallel
fi

# Increase the file descriptor limit
ulimit -n 65536

# Function to delete directories and files
delete_items() {
  local item_type=$1
  local name_pattern=$2

  echo -e "${yellow}Searching for ${name_pattern}...${nc}"
  found_items=$(find . -type ${item_type} -name "${name_pattern}")

  if [ -z "$found_items" ]; then
    echo -e "${red}No ${name_pattern} found.${nc}"
  else
    item_count=$(echo "$found_items" | wc -l)
    echo -e "${cyan}Items discovered: ${item_count}${nc}"
    echo "$found_items" | parallel --eta -j+0 --keep-order "rm -rf '{}'" |
      while read -r line; do
        clear
        echo -e "${yellow}Deleting items...${nc}"
        echo -e "${blue}${line}${nc}"
      done
    deleted_count=$(echo "$found_items" | wc -l)
    echo -e "${green}Items deleted: ${deleted_count}${nc}"
    echo -e "${green}${name_pattern} deleted successfully.${nc}"
  fi
}

# Delete all node_modules directories
delete_items d "node_modules"

# Delete all lock files (package-lock.json, pnpm-lock.yaml, yarn.lock)
delete_items f "package-lock.json"
delete_items f "pnpm-lock.yaml"
delete_items f "yarn.lock"

# Summary message
echo -e "${magenta}=========================================${nc}"
echo -e "${white}Cleanup Summary:${nc}"
echo -e "${green}All specified items have been deleted.${nc}"
echo -e "${magenta}=========================================${nc}"
