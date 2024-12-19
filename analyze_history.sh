#!/bin/bash

# Colors for better readability
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default configuration
HISTORY_DIR=".history"
SOURCE_DIR="src"
OUTPUT_DIR="docs"
FILE_TYPES=("coordinator" "sensor" "const")  # Default file types to analyze

# Help message
show_help() {
    echo "Code Historian - A tool for analyzing code history"
    echo
    echo "Usage: $0 [options]"
    echo
    echo "Options:"
    echo "  -h, --help                Show this help message"
    echo "  --history-dir DIR         Set history directory (default: .history)"
    echo "  --source-dir DIR          Set source directory (default: src)"
    echo "  --output-dir DIR          Set output directory (default: docs)"
    echo "  --files FILE1,FILE2,...   Specify files to analyze (without .py extension)"
    echo
    echo "Example:"
    echo "  $0 --history-dir .history --source-dir src --files model,view,controller"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --history-dir)
            HISTORY_DIR="$2"
            shift 2
            ;;
        --source-dir)
            SOURCE_DIR="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --files)
            IFS=',' read -ra FILE_TYPES <<< "$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Function to extract timestamp from filename
get_timestamp() {
    echo "$1" | grep -o "[0-9]\{14\}"
}

# Function to categorize a change based on content
categorize_change() {
    local content="$1"
    local categories=""
    
    # Look for specific patterns in the change
    if echo "$content" | grep -q "class.*:"; then
        categories+="[Class Structure] "
    fi
    if echo "$content" | grep -q "async.*def"; then
        categories+="[Async Methods] "
    fi
    if echo "$content" | grep -q "_LOGGER"; then
        categories+="[Logging] "
    fi
    if echo "$content" | grep -q "try:.*except"; then
        categories+="[Error Handling] "
    fi
    if echo "$content" | grep -q "@property"; then
        categories+="[Properties] "
    fi
    if echo "$content" | grep -q "TypedDict\|Optional\|List\|Dict"; then
        categories+="[Type Safety] "
    fi
    if echo "$content" | grep -q "self\._cleanup\|async_shutdown"; then
        categories+="[Resource Management] "
    fi
    if echo "$content" | grep -q "\"\"\".*\"\"\""; then
        categories+="[Documentation] "
    fi
    
    echo "$categories"
}

# Function to create a master knowledge base for a file
create_master_knowledge() {
    local file_type=$1
    local output_file="${OUTPUT_DIR}/master_changes_${file_type}.md"
    
    mkdir -p "$OUTPUT_DIR"
    
    echo "# Master Change History for ${file_type}.py" > "$output_file"
    echo "## Overview of All Changes" >> "$output_file"
    echo "This document tracks all significant changes made to ${file_type}.py, categorized by type and impact." >> "$output_file"
    echo >> "$output_file"
    
    # Find all history files for this type
    local history_files=$(find "$HISTORY_DIR" -type f -name "${file_type}_*.py" | sort)
    local prev_file=""
    
    echo "## Change Timeline" >> "$output_file"
    
    for file in $history_files; do
        local timestamp=$(get_timestamp "$file")
        local date_str=$(date -d "${timestamp:0:8} ${timestamp:8:6}" "+%Y-%m-%d %H:%M:%S" 2>/dev/null)
        
        if [ ! -z "$prev_file" ]; then
            echo -e "\n### Changes at $date_str" >> "$output_file"
            echo '```diff' >> "$output_file"
            
            # Get diff and analyze it
            local diff_content=$(diff -u "$prev_file" "$file")
            local categories=$(categorize_change "$diff_content")
            
            if [ ! -z "$categories" ]; then
                echo -e "\nCategories: $categories\n" >> "$output_file"
            fi
            
            echo "$diff_content" >> "$output_file"
            echo '```' >> "$output_file"
            
            # Extract and log significant changes
            if echo "$diff_content" | grep -q "^+.*class.*:"; then
                echo "- Added/Modified class definition" >> "$output_file"
            fi
            if echo "$diff_content" | grep -q "^+.*async.*def"; then
                echo "- Added/Modified async method" >> "$output_file"
            fi
            if echo "$diff_content" | grep -q "^+.*_LOGGER"; then
                echo "- Enhanced logging" >> "$output_file"
            fi
            if echo "$diff_content" | grep -q "^+.*try:.*except"; then
                echo "- Improved error handling" >> "$output_file"
            fi
        fi
        
        prev_file="$file"
    done
    
    # Compare with current version
    local current_file="${SOURCE_DIR}/custom_components/rtl433/${file_type}.py"
    if [ -f "$current_file" ] && [ ! -z "$prev_file" ]; then
        echo -e "\n### Current Version Changes" >> "$output_file"
        echo '```diff' >> "$output_file"
        diff -u "$prev_file" "$current_file" >> "$output_file"
        echo '```' >> "$output_file"
    fi
    
    # Add summary sections
    echo -e "\n## Key Improvements" >> "$output_file"
    echo "1. Type Safety Enhancements" >> "$output_file"
    echo "2. Error Handling Improvements" >> "$output_file"
    echo "3. Logging Enhancements" >> "$output_file"
    echo "4. Resource Management" >> "$output_file"
    echo "5. Documentation Updates" >> "$output_file"
    
    echo -e "\n## Lessons Learned" >> "$output_file"
    echo "Document contains key learnings from the development process." >> "$output_file"
    
    echo -e "${GREEN}Created master knowledge base for ${file_type}.py at ${output_file}${NC}"
}

# Function to analyze a specific file type
analyze_file() {
    local file_type=$1
    local current_file="${SOURCE_DIR}/custom_components/rtl433/${file_type}.py"
    
    echo -e "${BLUE}=== Analyzing ${file_type}.py ===${NC}"
    echo "Current file: ${current_file}"
    echo
    
    # Create master knowledge base
    create_master_knowledge "$file_type"
    
    # Get the most recent version from each path
    echo "Most recent versions in each path:"
    for path in "${HISTORY_DIR}/src/custom_components/rtl433" "${HISTORY_DIR}/custom_components/rtl433" "${HISTORY_DIR}/config/custom_components/rtl433"; do
        latest=$(find "$path" -type f -name "${file_type}_*.py" 2>/dev/null | sort -r | head -n 1)
        if [ ! -z "$latest" ]; then
            timestamp=$(get_timestamp "$latest")
            echo -e "${GREEN}  $path:${NC} $(date -d "${timestamp:0:8} ${timestamp:8:6}" "+%Y-%m-%d %H:%M:%S" 2>/dev/null) - $latest"
        fi
    done
    echo
}

# Main script
echo -e "${BLUE}Code Historian - History Analysis${NC}"
echo "================================="
echo

# Create example output if it doesn't exist
mkdir -p tools/code_historian/examples
if [ ! -f tools/code_historian/examples/sample_output.md ]; then
    echo "# Sample Code Historian Output" > tools/code_historian/examples/sample_output.md
    echo "This is an example of the output format..." >> tools/code_historian/examples/sample_output.md
fi

for file_type in "${FILE_TYPES[@]}"; do
    analyze_file "$file_type"
    echo
done

echo -e "${GREEN}Analysis complete! Check the ${OUTPUT_DIR}/ directory for detailed change history.${NC}" 