#!/bin/bash

# wl() {\n    if [ $# -lt 2 ]; then\n        echo "Usage: wl <filename> <text>"\n        echo "Appends text as a newline to the end of the specified file"\n        return 1\n    fi\n    local filename="$1"\n    shift\n    local text="$*"\n    local dir=$(dirname "$filename")\n    if [ ! -d "$dir" ]; then\n        echo "Error: Directory '$dir' does not exist"\n        return 1\n    fi\n    if [ -e "$filename" ] && [ ! -w "$filename" ]; then\n        echo "Error: Cannot write to '$filename' (permission denied)"\n        return 1\n    fi\n    echo "$text" >>"$filename"\n    if [ $? -eq 0 ]; then\n        echo "Successfully appended '$text' to '$filename'"\n    else\n        echo "Error: Failed to append to '$filename'"\n        return 1\n    fi\n}
wl() {
    if [ $# -lt 2 ]; then
        echo "Usage: wl <filename> <text>"
        echo "Appends text as a newline to the end of the specified file"
        return 1
    fi

    local filename="$1"
    shift           # Remove first argument
    local text="$*" # Join all remaining arguments as text

    # Check if directory exists
    local dir=$(dirname "$filename")
    if [ ! -d "$dir" ]; then
        echo "Error: Directory '$dir' does not exist"
        return 1
    fi

    # Check if file is writable (or can be created)
    if [ -e "$filename" ] && [ ! -w "$filename" ]; then
        echo "Error: Cannot write to '$filename' (permission denied)"
        return 1
    fi

    # Append text with newline
    echo "$text" >>"$filename"

    if [ $? -eq 0 ]; then
        echo "Successfully appended '$text' to '$filename'"
    else
        echo "Error: Failed to append to '$filename'"
        return 1
    fi
}
