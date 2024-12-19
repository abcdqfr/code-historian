#!/bin/bash

# Colors for better readability
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check for required dependencies
check_dependencies() {
    local missing_deps=()
    
    # Check for Python 3
    if ! command -v python3 &> /dev/null; then
        missing_deps+=("python3")
    fi
    
    # Check for pip
    if ! command -v pip3 &> /dev/null; then
        missing_deps+=("python3-pip")
    fi
    
    # Check for Graphviz system package
    if ! command -v dot &> /dev/null; then
        missing_deps+=("graphviz")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo -e "${RED}Missing required dependencies: ${missing_deps[*]}${NC}"
        echo "Please install them using your package manager:"
        echo "For Ubuntu/Debian:"
        echo "  sudo apt-get update"
        echo "  sudo apt-get install ${missing_deps[*]}"
        echo "For Fedora:"
        echo "  sudo dnf install ${missing_deps[*]}"
        echo "For macOS:"
        echo "  brew install ${missing_deps[*]}"
        exit 1
    fi
}

# Install Python dependencies
install_python_deps() {
    echo -e "${BLUE}Installing Python dependencies...${NC}"
    pip3 install graphviz --user
}

# Main installation
echo -e "${BLUE}Installing Code Historian...${NC}"

# Check dependencies
check_dependencies

# Install Python dependencies
install_python_deps

# Create installation directory
INSTALL_DIR="/usr/local/bin"
if [ ! -d "$INSTALL_DIR" ]; then
    echo -e "${BLUE}Creating installation directory...${NC}"
    sudo mkdir -p "$INSTALL_DIR"
fi

# Copy files
echo -e "${BLUE}Copying files...${NC}"
sudo cp code-historian "$INSTALL_DIR/"
sudo cp timeline_generator.py "$INSTALL_DIR/"

# Make executable
sudo chmod +x "$INSTALL_DIR/code-historian"
sudo chmod +x "$INSTALL_DIR/timeline_generator.py"

echo -e "${GREEN}Installation complete!${NC}"
echo "You can now use code-historian from anywhere."
echo
echo "Example usage:"
echo "  code-historian --files myfile --ext py --timeline"
echo "  code-historian --recursive --pattern '*.js' --timeline"