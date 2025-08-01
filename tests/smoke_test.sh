#!/bin/bash

# Nexus CLI Smoke Test
# This script runs the CLI in headless mode and kills it when proof submission is detected
# Usage: ./smoke_test.sh [binary_path] [node_id] [--once]
# Example: ./smoke_test.sh ./target/release/nexus-network 6166715 --once

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Global variable to track CLI process
CLI_PID=""

# Function to cleanup processes on script exit
cleanup() {
    if [ -n "$CLI_PID" ] && kill -0 $CLI_PID 2>/dev/null; then
        print_info "Cleaning up CLI process (PID: $CLI_PID)"
        kill -TERM $CLI_PID 2>/dev/null || true
        sleep 1
        kill -KILL $CLI_PID 2>/dev/null || true
        wait $CLI_PID 2>/dev/null || true
    fi
}

# Set up trap to cleanup on script exit
trap cleanup EXIT

echo -e "${YELLOW}Starting Nexus CLI Smoke Test...${NC}"

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

# Check for --once parameter (could be in position 2 or 3)
if [[ "$2" == "--once" ]] || [[ "$3" == "--once" ]]; then
    JUST_ONCE=true
    print_info "Running in --once mode - will exit after first proof or rate limiting"
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
    print_info "Example: $0 ./target/release/nexus-network 6166715"
    exit 1
fi

print_info "Using binary: $BINARY_PATH"
print_info "Using node IDs: ${NODE_IDS[*]}"
print_info "Monitoring for: $SUCCESS_PATTERN"

# Try each node ID until one works
for node_id in "${NODE_IDS[@]}"; do
    print_info "Trying node ID: $node_id"
    
    # Use a temporary file to capture output
    TEMP_OUTPUT=$(mktemp)
    trap "rm -f $TEMP_OUTPUT" EXIT

    # Start the CLI process in a subshell to suppress termination messages
    SUCCESS_FOUND=false
    RATE_LIMITED=false
    EXIT_EARLY=false
    
    (
        "$BINARY_PATH" start --headless --node-id $node_id 2>&1 | tee "$TEMP_OUTPUT" &
        CLI_PID=$!
        
        # Monitor the output file for the success pattern
        for i in $(seq 1 $MAX_TIMEOUT_SECONDS); do
            # Check for actual success patterns first
            if grep -q "$SUCCESS_PATTERN" "$TEMP_OUTPUT" 2>/dev/null; then
                print_status "Success pattern detected: $SUCCESS_PATTERN"
                SUCCESS_FOUND=true
                break
            fi
            
            # Check for rate limiting (but don't stop - keep trying)
            if [ "$SUCCESS_FOUND" = false ]; then
                if grep -q "Rate limited" "$TEMP_OUTPUT" 2>/dev/null; then
                    if [ "$RATE_LIMITED" = false ]; then
                        print_info "Rate limited detected - continuing to wait for tasks..."
                        RATE_LIMITED=true
                    fi
                    
                    # In --once mode, exit immediately when rate limited
                    if [ "$JUST_ONCE" = true ]; then
                        print_info "Rate limited in --once mode, trying next node ID"
                        EXIT_EARLY=true
                        break
                    fi
                fi
            fi
            
            sleep 1
        done
        
        # Clean up the process
        if [ -n "$CLI_PID" ] && kill -0 $CLI_PID 2>/dev/null; then
            kill -TERM $CLI_PID 2>/dev/null || true
            sleep 2
            kill -KILL $CLI_PID 2>/dev/null || true
            wait $CLI_PID 2>/dev/null || true
        fi
    ) 2>/dev/null

    # Process will be cleaned up by the trap on exit
    CLI_PID=""  # Clear the global variable so trap doesn't try to kill again

    # Check if we found the success pattern
    if [ "$SUCCESS_FOUND" = true ]; then
        print_status "Smoke test PASSED - CLI successfully submitted proof with node ID: $node_id"
        if [ "$JUST_ONCE" = true ]; then
            exit 0
        fi
    else
        print_info "No success with node ID: $node_id"
        if [ "$RATE_LIMITED" = true ]; then
            print_info "Note: CLI was rate limited during the test period"
        fi
        
        # In --once mode, continue to next node ID if rate limited
        if [ "$JUST_ONCE" = true ] && [ "$EXIT_EARLY" = true ]; then
            print_info "Rate limited with node ID $node_id, trying next node ID..."
            continue
        fi
    fi
    
    # Clean up temp file
    rm -f "$TEMP_OUTPUT"
done

# If we get here, none of the node IDs worked
print_error "Smoke test FAILED - No proof submission detected with any node ID within $MAX_TIMEOUT_SECONDS seconds"
echo "CLI Output from last attempt:"
cat "$TEMP_OUTPUT"
echo ""
print_info "Checked for success patterns:"
echo "  - $SUCCESS_PATTERN"
print_info "Tried node IDs: ${NODE_IDS[*]}"
exit 1 