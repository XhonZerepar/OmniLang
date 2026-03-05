#!/bin/bash
# OmniLang Ecosystem Installer
# Installs the OmniLang ecosystem tools and dependencies

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
OMNI_VERSION="0.3.0"
OMNI_HOME="${OMNI_HOME:-$HOME/.omni}"
INSTALL_DIR="$OMNI_HOME"
BIN_DIR="$INSTALL_DIR/bin"
LIB_DIR="$INSTALL_DIR/lib"
CACHE_DIR="$INSTALL_DIR/cache"

echo -e "${BLUE}OmniLang Ecosystem Installer${NC}"
echo "==============================="
echo ""

# Check for required tools
check_command() {
    if ! command -v "$1" &> /dev/null; then
        echo -e "${RED}Error: $1 is required but not installed.${NC}"
        exit 1
    fi
}

echo -e "${YELLOW}Checking prerequisites...${NC}"
check_command "python3"
check_command "curl"
check_command "tar"

# Check Python version
PYTHON_VERSION=$(python3 --version | cut -d' ' -f2 | cut -d'.' -f1,2)
if [ "$(echo "$PYTHON_VERSION < 3.7" | bc)" -eq 1 ]; then
    echo -e "${RED}Error: Python 3.7 or higher is required.${NC}"
    exit 1
fi

# Check for TOML support
if ! python3 -c "import toml" 2>/dev/null; then
    echo -e "${YELLOW}Installing toml module...${NC}"
    pip3 install toml
fi

echo -e "${GREEN}Prerequisites OK!${NC}"
echo ""

# Create directories
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p "$BIN_DIR"
mkdir -p "$LIB_DIR"
mkdir -p "$CACHE_DIR"
mkdir -p "$LIB_DIR/std"
mkdir -p "$LIB_DIR/tools"
mkdir -p "$LIB_DIR/templates"
echo -e "${GREEN}Directories created!${NC}"
echo ""

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Install CLI
echo -e "${YELLOW}Installing Omni CLI...${NC}"
cp "$SCRIPT_DIR/omni" "$BIN_DIR/omni"
chmod +x "$BIN_DIR/omni"
echo -e "${GREEN}CLI installed to $BIN_DIR/omni${NC}"
echo ""

# Install standard library
echo -e "${YELLOW}Installing Standard Library...${NC}"
cp -r "$SCRIPT_DIR/stdlib/"* "$LIB_DIR/std/"
echo -e "${GREEN}Standard library installed!${NC}"
echo ""

# Install tools
echo -e "${YELLOW}Installing Tools...${NC}"
cp -r "$SCRIPT_DIR/tools/"* "$LIB_DIR/tools/"
echo -e "${GREEN}Tools installed!${NC}"
echo ""

# Install templates
echo -e "${YELLOW}Installing Project Templates...${NC}"
cp -r "$SCRIPT_DIR/templates/"* "$LIB_DIR/templates/"
echo -e "${GREEN}Templates installed!${NC}"
echo ""

# Create configuration
echo -e "${YELLOW}Creating configuration...${NC}"

cat > "$INSTALL_DIR/config.toml" <<EOF
[omni]
version = "$OMNI_VERSION"
home = "$INSTALL_DIR"

[paths]
bin = "$BIN_DIR"
lib = "$LIB_DIR"
cache = "$CACHE_DIR"

[paths.stdlib]
fs = "$LIB_DIR/std/fs.omni"
http = "$LIB_DIR/std/http.omni"
time = "$LIB_DIR/std/time.omni"
test = "$LIB_DIR/std/test.omni"
pkg = "$LIB_DIR/std/pkg.omni"

[paths.templates]
basic = "$LIB_DIR/templates/basic"
web = "$LIB_DIR/templates/web"
lib = "$LIB_DIR/templates/lib"
cli = "$LIB_DIR/templates/cli"

[paths.tools]
doc = "$LIB_DIR/tools/omnidoc.omni"

[build]
optimization = "default"
target = "native"
EOF

echo -e "${GREEN}Configuration created!${NC}"
echo ""

# Create environment setup script
echo -e "${YELLOW}Creating environment setup...${NC}"

cat > "$INSTALL_DIR/env.sh" <<EOF
# OmniLang Environment
export OMNI_HOME="$INSTALL_DIR"
export PATH="\$OMNI_HOME/bin:\$PATH"
export OMNI_LIB="$LIB_DIR"
export OMNI_CACHE="$CACHE_DIR"
EOF

echo -e "${GREEN}Environment script created!${NC}"
echo ""

# Add to PATH permanently
add_to_path() {
    local shell_rc=""
    
    if [ -n "$ZSH_VERSION" ]; then
        shell_rc="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ]; then
        shell_rc="$HOME/.bashrc"
    fi
    
    if [ -n "$shell_rc" ]; then
        if ! grep -q "omni.env.sh" "$shell_rc" 2>/dev/null;
then
            echo "" >> "$shell_rc"
            echo "# OmniLang Environment" >> "$shell_rc"
            echo "source \"$INSTALL_DIR/env.sh\"" >> "$shell_rc"
            echo -e "${GREEN}Added to $shell_rc${NC}"
        fi
    fi
}

echo -e "${YELLOW}Would you like to add OmniLang to your PATH? [y/N]${NC}"
read -r response

if [ "$response" = "y" ] || [ "$response" = "Y" ]; then
    add_to_path
else
    echo -e "${YELLOW}Skipped. Add manually:${NC}"
    echo "  source $INSTALL_DIR/env.sh"
fi

echo ""
echo -e "${GREEN}==========================================${NC}"
echo -e "${GREEN}Installation complete!${NC}"
echo -e "${GREEN}==========================================${NC}"
echo ""
echo "To get started:"
echo ""
echo "  1. Source the environment:"
echo "     source $INSTALL_DIR/env.sh"
echo ""
echo "  2. Test the CLI:"
echo "     omni version"
echo ""
echo "  3. Create a new project:"
echo "     omni init my-project"
echo ""
echo "  4. Build and run:"
echo "     cd my-project"
echo "     omni run"
echo ""
