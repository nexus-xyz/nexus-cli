#!/bin/sh

# -----------------------------------------------------------------------------
# 1) Define environment variables and colors for terminal output.
# -----------------------------------------------------------------------------
NEXUS_HOME="$HOME/.nexus"
BIN_DIR="$NEXUS_HOME/bin"
GREEN='\033[1;32m'
ORANGE='\033[1;33m'
RED='\033[1;31m'
NC='\033[0m'  # No Color

# Ensure the $NEXUS_HOME and $BIN_DIR directories exist.
[ -d "$NEXUS_HOME" ] || mkdir -p "$NEXUS_HOME"
[ -d "$BIN_DIR" ] || mkdir -p "$BIN_DIR"

# -----------------------------------------------------------------------------
# 2) Display a message if we're interactive (NONINTERACTIVE is not set) and the
#    $NODE_ID is not a 28-character ID. This is for Testnet II info.
# -----------------------------------------------------------------------------
if [ -z "$NONINTERACTIVE" ] && [ "${#NODE_ID}" -ne "28" ]; then
    echo ""
    echo "${GREEN}Testnet III is now live!${NC}"
    echo ""
fi

# -----------------------------------------------------------------------------
# 3) Prompt the user to agree to the Nexus Beta Terms of Use if we're in an
#    interactive mode (i.e., NONINTERACTIVE is not set) and no config.json file exists.
#    We explicitly read from /dev/tty to ensure user input is requested from the
#    terminal rather than the script's standard input.
# -----------------------------------------------------------------------------
while [ -z "$NONINTERACTIVE" ] && [ ! -f "$NEXUS_HOME/config.json" ]; do
    read -p "Do you agree to the Nexus Beta Terms of Use (https://nexus.xyz/terms-of-use)? (Y/n) " yn </dev/tty
    echo ""

    case $yn in
        [Nn]* )
            echo ""
            exit;;
        [Yy]* )
            echo ""
            break;;
        "" )
            echo ""
            break;;
        * )
            echo "Please answer yes or no."
            echo "";;
    esac
done

# -----------------------------------------------------------------------------
# 4) Determine the platform and architecture
# -----------------------------------------------------------------------------
case "$(uname -s)" in
    Linux*)
        PLATFORM="linux"
        case "$(uname -m)" in
            x86_64)
                ARCH="x86_64"
                BINARY_NAME="nexus-network-linux-x86_64"
                ;;
            aarch64|arm64)
                ARCH="arm64"
                BINARY_NAME="nexus-network-linux-arm64"
                ;;
            *)
                echo "${RED}Unsupported architecture: $(uname -m)${NC}"
                echo "Please build from source:"
                echo "  git clone https://github.com/nexus-xyz/nexus-cli.git"
                echo "  cd nexus-cli/clients/cli"
                echo "  cargo build --release"
                exit 1
                ;;
        esac
        ;;
    Darwin*)
        PLATFORM="macos"
        case "$(uname -m)" in
            x86_64)
                ARCH="x86_64"
                BINARY_NAME="nexus-network-macos-x86_64"
                echo "${ORANGE}Note: You are running on an Intel Mac.${NC}"
                ;;
            arm64)
                ARCH="arm64"
                BINARY_NAME="nexus-network-macos-arm64"
                echo "${ORANGE}Note: You are running on an Apple Silicon Mac (M1/M2/M3).${NC}"
                ;;
            *)
                echo "${RED}Unsupported architecture: $(uname -m)${NC}"
                echo "Please build from source:"
                echo "  git clone https://github.com/nexus-xyz/nexus-cli.git"
                echo "  cd nexus-cli/clients/cli"
                echo "  cargo build --release"
                exit 1
                ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM="windows"
        case "$(uname -m)" in
            x86_64)
                ARCH="x86_64"
                BINARY_NAME="nexus-network-windows-x86_64.exe"
                ;;
            *)
                echo "${RED}Unsupported architecture: $(uname -m)${NC}"
                echo "Please build from source:"
                echo "  git clone https://github.com/nexus-xyz/nexus-cli.git"
                echo "  cd nexus-cli/clients/cli"
                echo "  cargo build --release"
                exit 1
                ;;
        esac
        ;;
    *)
        echo "${RED}Unsupported platform: $(uname -s)${NC}"
        echo "Please build from source:"
        echo "  git clone https://github.com/nexus-xyz/nexus-cli.git"
        echo "  cd nexus-cli/clients/cli"
        echo "  cargo build --release"
        exit 1
        ;;
esac

# -----------------------------------------------------------------------------
# 5) Download latest release binary
# -----------------------------------------------------------------------------
echo "Fetching latest release information..."

# First, fetch the release data and check if we got a valid response
RELEASE_DATA=$(curl -s --connect-timeout 10 --max-time 30 https://api.github.com/repos/nexus-xyz/nexus-cli/releases/latest)

if [ $? -ne 0 ] || [ -z "$RELEASE_DATA" ]; then
    echo "${RED}Failed to fetch release information from GitHub API${NC}"
    echo "Please try again or build from source:"
    echo "  git clone https://github.com/nexus-xyz/nexus-cli.git"
    echo "  cd nexus-cli/clients/cli"
    echo "  cargo build --release"
    exit 1
fi

# Check if the response contains valid JSON (should have "assets" field)
if ! echo "$RELEASE_DATA" | grep -q '"assets"'; then
    echo "${RED}Invalid response from GitHub API${NC}"
    echo "Please try again or build from source:"
    echo "  git clone https://github.com/nexus-xyz/nexus-cli.git"
    echo "  cd nexus-cli/clients/cli"
    echo "  cargo build --release"
    exit 1
fi

# We need to match the exact name, not just a substring
LATEST_RELEASE_URL=$(echo "$RELEASE_DATA" | \
    tr ',' '\n' | \
    awk -v target="$BINARY_NAME" '
    BEGIN { found=0; url="" }
    /"name":"/ {
        gsub(/.*"name":"/, "", $0)
        gsub(/".*/, "", $0)
        if ($0 == target) found=1
        else found=0
    }
    found && /"browser_download_url":"/ {
        gsub(/.*"browser_download_url":"/, "", $0)
        gsub(/".*/, "", $0)
        url = $0
        found=0
    }
    END { print url }
    ')

if [ -z "$LATEST_RELEASE_URL" ]; then
    echo "${RED}Could not find a precompiled binary for $PLATFORM-$ARCH (looking for: $BINARY_NAME)${NC}"
    echo "This might indicate that no release exists for your platform or there was an issue parsing the release data."
    echo "Please build from source:"
    echo "  git clone https://github.com/nexus-xyz/nexus-cli.git"
    echo "  cd nexus-cli/clients/cli"
    echo "  cargo build --release"
    exit 1
fi

echo "Downloading latest release for ${GREEN}$PLATFORM-$ARCH${NC}..."
echo "Downloading from: ${GREEN}$LATEST_RELEASE_URL${NC}"
curl -L -o "$BIN_DIR/nexus-network" "$LATEST_RELEASE_URL"
chmod +x "$BIN_DIR/nexus-network"
ln -s "$BIN_DIR/nexus-network" "$BIN_DIR/nexus-cli"
chmod +x "$BIN_DIR/nexus-cli"

# -----------------------------------------------------------------------------
# 6) Add $BIN_DIR to PATH if not already present
# -----------------------------------------------------------------------------
case "$SHELL" in
    */bash)
        PROFILE_FILE="$HOME/.bashrc"
        ;;
    */zsh)
        PROFILE_FILE="$HOME/.zshrc"
        ;;
    *)
        PROFILE_FILE="$HOME/.profile"
        ;;
esac

# Only append if not already in PATH
if ! echo "$PATH" | grep -q "$BIN_DIR"; then
    if ! grep -qs "$BIN_DIR" "$PROFILE_FILE"; then
        echo "" >> "$PROFILE_FILE"
        echo "# Add Nexus CLI to PATH" >> "$PROFILE_FILE"
        echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$PROFILE_FILE"
        echo "${GREEN}Updated PATH in $PROFILE_FILE${NC}"
    fi
fi

echo ""
echo "${GREEN}Installation complete!${NC}"
echo "Restart your terminal or run the following command to update your PATH:"
echo "  source $PROFILE_FILE"
echo ""
echo "${ORANGE}To get your node ID, visit: https://app.nexus.xyz/nodes${NC}"
echo ""
echo "Register your user to begin linked proving with the Nexus CLI by: nexus-cli register-user --wallet-address <WALLET_ADDRESS>"
echo "Or follow the guide at https://docs.nexus.xyz/layer-1/testnet/cli-node"
