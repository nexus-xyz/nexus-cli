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
NODE_ID="${2:-6166715}" # Use second argument or default (fallback)
MAX_TIMEOUT_SECONDS=180 # 3 minutes max timeout
SUCCESS_PATTERN="Step 4 of 4: Proof submitted successfully"
JUST_ONCE=false

# Check for --max-tasks parameter (could be in position 2 or 3)
if [[ "$2" == "--max-tasks" ]] || [[ "$3" == "--max-tasks" ]]; then
	JUST_ONCE=true
	print_info "Running with max-tasks=1 - will exit after first proof or rate limiting"
fi

# Parse node IDs from environment variable (GitHub secret) or use fallback
if [ -n "$SMOKE_TEST_NODE_IDS" ]; then
	# Split comma-separated string into array
	IFS=',' read -ra NODE_IDS <<<"$SMOKE_TEST_NODE_IDS"
elif [ "$2" != "--max-tasks" ] && [ -n "$2" ]; then
	# If a specific node ID was provided as argument, use only that one
	NODE_IDS=("$2")
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

	# Use temporary files to capture output
	TEMP_OUTPUT=$(mktemp)
	TEMP_RAW_OUTPUT=$(mktemp)
	trap "rm -f $TEMP_OUTPUT $TEMP_RAW_OUTPUT" EXIT

	# Start the CLI process and capture output with timeout
	print_info "Starting CLI process..."

	# Start the CLI process in background (release mode should have clean output)
	(
		ulimit -c 0
		RUST_LOG=warn "$BINARY_PATH" start --headless --max-tasks 1 --node-id $node_id 2>&1 | tee "$TEMP_RAW_OUTPUT"
	) &
	CLI_PID=$!

	# Wait for either completion or timeout (60 seconds)
	TIMEOUT=60
	for i in $(seq 1 $TIMEOUT); do
		if ! kill -0 "$CLI_PID" 2>/dev/null; then
			# Process has finished
			wait "$CLI_PID"
			CLI_EXIT_CODE=$?
			break
		fi

		# Check for success pattern every 5 seconds and show progress
		if [ $((i % 10)) -eq 0 ]; then
			print_info "CLI still running... ($i/$TIMEOUT seconds)"
			if [ -f "$TEMP_RAW_OUTPUT" ]; then
				# Show the last few lines to see progress
				LAST_LINES=$(tail -3 "$TEMP_RAW_OUTPUT" 2>/dev/null)
				if [ -n "$LAST_LINES" ]; then
					print_info "Recent activity:"
					echo "$LAST_LINES" | while IFS= read -r line; do
						echo "    $line"
					done
				fi
			fi
		fi

		if [ $((i % 5)) -eq 0 ] && [ -f "$TEMP_RAW_OUTPUT" ]; then
			if grep -q "$SUCCESS_PATTERN" "$TEMP_RAW_OUTPUT" 2>/dev/null; then
				print_status "Success pattern detected early, waiting for clean exit..."
				# Give it 30 more seconds to exit cleanly
				for j in $(seq 1 30); do
					if ! kill -0 "$CLI_PID" 2>/dev/null; then
						print_info "CLI exited cleanly after success"
						wait "$CLI_PID" 2>/dev/null
						CLI_EXIT_CODE=$?
						break 2
					fi
					sleep 1
				done
				# If still running after 30 seconds, terminate it
				if kill -0 "$CLI_PID" 2>/dev/null; then
					print_info "CLI still running after 30s, terminating..."
					kill -TERM "$CLI_PID" 2>/dev/null
					sleep 2
					if kill -0 "$CLI_PID" 2>/dev/null; then
						kill -KILL "$CLI_PID" 2>/dev/null
					fi
					wait "$CLI_PID" 2>/dev/null
					CLI_EXIT_CODE=$?
				fi
				break
			fi
		fi

		sleep 1
	done

	# If we reached timeout, kill the process
	if kill -0 "$CLI_PID" 2>/dev/null; then
		print_info "CLI process timed out after $TIMEOUT seconds, terminating..."
		kill -TERM "$CLI_PID" 2>/dev/null
		sleep 2
		if kill -0 "$CLI_PID" 2>/dev/null; then
			kill -KILL "$CLI_PID" 2>/dev/null
		fi
		wait "$CLI_PID" 2>/dev/null
		CLI_EXIT_CODE=$?
	fi

	if [ "$CLI_EXIT_CODE" -eq 0 ]; then
		# Process completed successfully
		print_info "CLI process completed successfully"

		if grep -q "$SUCCESS_PATTERN" "$TEMP_RAW_OUTPUT" 2>/dev/null; then
			print_status "Success pattern detected: $SUCCESS_PATTERN"
			SUCCESS_FOUND=true
		else
			print_info "No success pattern found in output"
		fi
	else
		# Process failed or was terminated
		if [ "$CLI_EXIT_CODE" -eq 143 ]; then
			# Process was terminated by SIGTERM (timeout or signal)
			print_info "Process terminated by signal"
		elif [ "$CLI_EXIT_CODE" -eq 124 ]; then
			# Process timed out
			print_info "Process timed out"
		else
			print_info "Process exited with code: $CLI_EXIT_CODE"
		fi

		if grep -q "Rate limited" "$TEMP_RAW_OUTPUT" 2>/dev/null; then
			RATE_LIMITED=true
		fi
	fi

	# Show last few lines of CLI output for debugging
	print_info "CLI output (last 10 lines):"
	tail -10 "$TEMP_RAW_OUTPUT" 2>/dev/null | while IFS= read -r line; do
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

	# Clean up temp files
	rm -f "$TEMP_OUTPUT" "$TEMP_RAW_OUTPUT"
done

# If we get here, none of the node IDs worked
print_error "Integration test FAILED - No proof submission detected within $MAX_TIMEOUT_SECONDS seconds"
print_info "Checked for success patterns:"
echo "  - $SUCCESS_PATTERN"
exit 1
