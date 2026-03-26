#!/bin/bash

# Colors for output
green='\033[0;32m'
red='\033[0;31m'
yellow='\033[1;33m'
blue='\033[1;34m'
cyan='\033[1;36m'
nc='\033[0m' # No Color

# Function to delete directories and files
delete_items() {
  local item_type=$1
  local name_pattern=$2

  echo -e "${yellow}Searching for ${name_pattern}...${nc}"
  found_items=$(find . -type ${item_type} -name "${name_pattern}")

  if [ -z "$found_items" ]; then
    echo -e "${red}No ${name_pattern} found.${nc}"
  else
    echo "$found_items" | while read -r item; do
      echo -e "${blue}Deleting: ${cyan}${item}${nc}"
      rm -rf "$item"
    done
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
echo -e "${green}Cleanup completed!${nc}"
