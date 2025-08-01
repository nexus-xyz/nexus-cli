#!/bin/bash

# Nexus CLI Smoke Test
# This script runs the CLI in headless mode and kills it when proof submission is detected
# Usage: ./smoke_test.sh [binary_path] [node_id]
# Example: ./smoke_test.sh ./target/release/nexus-network 6166715

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Starting Nexus CLI Smoke Test...${NC}"

# Configuration
NODE_ID="${2:-6166715}"  # Use second argument or default
MAX_TIMEOUT_SECONDS=180  # 3 minutes max timeout
SUCCESS_PATTERNS=(
    "Proof submitted"
    "Task step 3 of 3"
    "Points for this node will be updated"
    "Task completed successfully"
    "Proof hash generated"
    "Worker.*submitted proof"
    "Rate limited"
    "Fetching task"
    "Queue status"
    "Task Fetcher"
)

# Function to print colored output
print_status() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

# Get binary path from command line argument or use default
BINARY_PATH="${1:-./target/release/nexus-network}"

# Check if the CLI binary exists
if [ ! -f "$BINARY_PATH" ]; then
    print_error "CLI binary not found at: $BINARY_PATH"
    print_info "Usage: $0 [binary_path] [node_id]"
    print_info "Example: $0 ./target/release/nexus-network 6166715"
    exit 1
fi

print_info "Using binary: $BINARY_PATH"
print_info "Using node ID: $NODE_ID"
print_info "Monitoring for: $SUCCESS_PATTERNS"

# Start the CLI in headless mode and monitor output
# Use a temporary file to capture output
TEMP_OUTPUT=$(mktemp)
trap "rm -f $TEMP_OUTPUT" EXIT

# Start the CLI process
"$BINARY_PATH" start --headless --node-id $NODE_ID 2>&1 | tee "$TEMP_OUTPUT" &
CLI_PID=$!

# Monitor the output file for the success pattern
SUCCESS_FOUND=false
for i in $(seq 1 $MAX_TIMEOUT_SECONDS); do
    for pattern in "${SUCCESS_PATTERNS[@]}"; do
        if grep -q "$pattern" "$TEMP_OUTPUT" 2>/dev/null; then
            print_status "Success pattern detected: $pattern"
            SUCCESS_FOUND=true
            break 2 # Break both loops
        fi
    done
    sleep 1
done

# Kill the CLI process
kill $CLI_PID 2>/dev/null || true
wait $CLI_PID 2>/dev/null || true

# Check if we found the success pattern
if [ "$SUCCESS_FOUND" = true ]; then
    print_status "Smoke test PASSED - CLI successfully submitted proof"
    exit 0
else
    print_error "Smoke test FAILED - No proof submission detected within $MAX_TIMEOUT_SECONDS seconds"
    echo "CLI Output:"
    cat "$TEMP_OUTPUT"
    echo ""
    print_info "Checked for patterns:"
    for pattern in "${SUCCESS_PATTERNS[@]}"; do
        echo "  - $pattern"
    done
    exit 1
fi 