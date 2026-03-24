#!/bin/bash
# setup_doublezero.sh - Integration of DoubleZero for Arkhe OS
set -e

NETWORK=${1:-testnet} # Default to testnet
DZ_VERSION="latest"

echo "🌐 DoubleZero Setup for Arkhe OS ($NETWORK)"
echo "==========================================="

# 1. Install DoubleZero Packages
if [ -f /etc/debian_version ]; then
    echo "📦 Detected Debian/Ubuntu based system"
    if [ "$NETWORK" == "mainnet-beta" ]; then
        REPO_URL="https://dl.cloudsmith.io/public/malbeclabs/doublezero/setup.deb.sh"
    else
        REPO_URL="https://dl.cloudsmith.io/public/malbeclabs/doublezero-testnet/setup.deb.sh"
    fi
    curl -1sLf "$REPO_URL" | sudo -E bash
    sudo apt-get install -y doublezero
elif [ -f /etc/redhat-release ]; then
    echo "📦 Detected RHEL/Rocky based system"
    if [ "$NETWORK" == "mainnet-beta" ]; then
        REPO_URL="https://dl.cloudsmith.io/public/malbeclabs/doublezero/setup.rpm.sh"
    else
        REPO_URL="https://dl.cloudsmith.io/public/malbeclabs/doublezero-testnet/setup.rpm.sh"
    fi
    curl -1sLf "$REPO_URL" | sudo -E bash
    sudo yum install -y doublezero
else
    echo "❌ Unsupported OS for automated installation."
    exit 1
fi

# 2. Configure Firewall for GRE and BGP
echo "🛡️ Configuring Firewall..."
sudo iptables -A INPUT -p gre -j ACCEPT
sudo iptables -A OUTPUT -p gre -j ACCEPT
sudo iptables -A INPUT -i doublezero0 -s 169.254.0.0/16 -d 169.254.0.0/16 -p tcp --dport 179 -j ACCEPT
sudo iptables -A OUTPUT -o doublezero0 -s 169.254.0.0/16 -d 169.254.0.0/16 -p tcp --dport 179 -j ACCEPT

# 3. Create DoubleZero Identity
if [ ! -f ~/.config/doublezero/id.json ]; then
    echo "🔑 Generating New DoubleZero Identity..."
    doublezero keygen
else
    echo "🔑 DoubleZero Identity already exists."
fi

# 4. Show Address
echo "📍 DoubleZero Address:"
doublezero address

# 5. Check discovery
echo "📡 Checking discovery (latency)..."
doublezero latency || echo "Waiting for discovery..."

echo "✅ DoubleZero setup complete."
