#!/bin/bash

# Nexus CLI Integration Test
# This script runs the CLI in headless mode and verifies it can submit proofs to production
# Usage: ./integration_test.sh [binary_path] [node_id] [--max-tasks]
# Example: ./integration_test.sh ./target/release/nexus-network 6166715 --max-tasks 1

set -e

# Disable core dumps globally
ulimit -c 0

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color



echo -e "${YELLOW}Starting Nexus CLI Integration Test...${NC}"

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

# Configuration
NODE_ID="${2:-6166715}"  # Use second argument or default (fallback)
MAX_TIMEOUT_SECONDS=180  # 3 minutes max timeout
SUCCESS_PATTERN="Proof submitted"
JUST_ONCE=false

# Check for --max-tasks parameter (could be in position 2 or 3)
if [[ "$2" == "--max-tasks" ]] || [[ "$3" == "--max-tasks" ]]; then
    JUST_ONCE=true
    print_info "Running with max-tasks=1 - will exit after first proof or rate limiting"
fi

# Parse node IDs from environment variable (GitHub secret) or use fallback
if [ -n "$SMOKE_TEST_NODE_IDS" ]; then
    # Split comma-separated string into array
    IFS=',' read -ra NODE_IDS <<< "$SMOKE_TEST_NODE_IDS"
else
    # Fallback node IDs if secret not available
    NODE_IDS=(
        "6166715"
        "23716208"
        "23519580"
        "23718361"
    )
fi

# Get binary path from command line argument or use default
BINARY_PATH="${1:-./target/release/nexus-network}"

# Check if the CLI binary exists
if [ ! -f "$BINARY_PATH" ]; then
    print_error "CLI binary not found at: $BINARY_PATH"
    print_info "Usage: $0 [binary_path] [node_id]"
    print_info "Example: $0 ./target/release/nexus-network <node_id>"
    exit 1
fi

print_info "Using binary: $BINARY_PATH"
print_info "Monitoring for: $SUCCESS_PATTERN"

# Shuffle the node IDs array to load balance
NODE_IDS=($(printf '%s\n' "${NODE_IDS[@]}" | sort -R))

# Try each node ID until one works
for node_id in "${NODE_IDS[@]}"; do
    
    # Use a temporary file to capture output
    TEMP_OUTPUT=$(mktemp)
    trap "rm -f $TEMP_OUTPUT" EXIT

    # Start the CLI process and capture output
    print_info "Starting CLI process..."
    if (ulimit -c 0; RUST_LOG=warn "$BINARY_PATH" start --headless --max-tasks 1 --node-id $node_id 2>&1 | tee "$TEMP_OUTPUT"); then
        # Process completed successfully
        print_info "CLI process completed successfully"
        if grep -q "$SUCCESS_PATTERN" "$TEMP_OUTPUT" 2>/dev/null; then
            print_status "Success pattern detected: $SUCCESS_PATTERN"
            SUCCESS_FOUND=true
        else
            print_info "No success pattern found in output"
        fi
    else
        # Process failed or was terminated
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 143 ]; then
            # Process was terminated by SIGTERM (timeout or signal)
            print_info "Process terminated by signal"
        elif [ $EXIT_CODE -eq 124 ]; then
            # Process timed out
            print_info "Process timed out"
        else
            print_info "Process exited with code: $EXIT_CODE"
        fi
        
        if grep -q "Rate limited" "$TEMP_OUTPUT" 2>/dev/null; then
            RATE_LIMITED=true
        fi
    fi
    
    # Show last few lines of CLI output for debugging
    print_info "CLI output (last 10 lines):"
    tail -10 "$TEMP_OUTPUT" | while IFS= read -r line; do
        echo "  $line"
    done

    # Check if we found the success pattern
    if [ "$SUCCESS_FOUND" = true ]; then
        print_status "Integration test PASSED - CLI successfully submitted proof"
        exit 0
    elif [ "$JUST_ONCE" = true ] && [ "$EXIT_EARLY" = true ]; then
        # In --once mode, continue to next node ID if rate limited
        continue
    else
        if [ "$RATE_LIMITED" = true ]; then
            print_info "Rate limited"
        fi
    fi
    
    # Clean up temp file
    rm -f "$TEMP_OUTPUT"
done

# If we get here, none of the node IDs worked
print_error "Integration test FAILED - No proof submission detected within $MAX_TIMEOUT_SECONDS seconds"
print_info "Checked for success patterns:"
echo "  - $SUCCESS_PATTERN"
exit 1 