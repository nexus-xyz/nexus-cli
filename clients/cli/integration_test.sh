#!/bin/bash

# Nexus CLI Integration Test (run from clients/cli)
# Verifies the CLI can submit a proof to production
# Usage: ./integration_test.sh [binary_path] [node_id] [--max-tasks]

set -e

ulimit -c 0 || true

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Starting Nexus CLI Integration Test...${NC}"

print_status() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
print_error()  { echo -e "${RED}[ERROR]${NC} $1"; }
print_info()   { echo -e "${YELLOW}[INFO]${NC} $1"; }

NODE_ID="${2:-6166715}"
JUST_ONCE=false
if [[ "$2" == "--max-tasks" ]] || [[ "$3" == "--max-tasks" ]]; then
	JUST_ONCE=true
	print_info "Running with max-tasks=1 - will exit after first proof or rate limiting"
fi

NODE_IDS_ENV="${INTEGRATION_TEST_NODE_IDS}"
if [ -n "$NODE_IDS_ENV" ]; then
	IFS=',' read -ra NODE_IDS <<<"$NODE_IDS_ENV"
else
	NODE_IDS=("6166715" "23716208" "23519580" "23718361")
fi

BINARY_PATH="${1:-./target/release/nexus-network}"
if [ ! -f "$BINARY_PATH" ]; then
	print_error "CLI binary not found at: $BINARY_PATH"
	exit 1
fi

print_info "Using binary: $BINARY_PATH"
print_info "Using configured node IDs (retry at most once on rate limit)"

if command -v shuf >/dev/null 2>&1; then
	NODE_IDS=($(printf '%s\n' "${NODE_IDS[@]}" | shuf))
fi

ATTEMPT_COUNT=0

for node_id in "${NODE_IDS[@]}"; do
	if [ $ATTEMPT_COUNT -ge 2 ]; then
		break
	fi
	ATTEMPT_COUNT=$((ATTEMPT_COUNT+1))

	TEMP_RAW_OUTPUT=$(mktemp)
	trap "rm -f $TEMP_RAW_OUTPUT" EXIT
	print_info "Starting CLI process (attempt $ATTEMPT_COUNT) with node $node_id..."

	RATE_LIMITED_THIS_ATTEMPT=false
	(
		ulimit -c 0 || true
		RUST_LOG=warn "$BINARY_PATH" start --headless --max-tasks 1 --node-id $node_id 2>&1 | tee "$TEMP_RAW_OUTPUT"
	) &
	CLI_PID=$!

	TIMEOUT=60
	for i in $(seq 1 $TIMEOUT); do
		if ! kill -0 "$CLI_PID" 2>/dev/null; then
			wait "$CLI_PID"; CLI_EXIT_CODE=$?; break
		fi
		if [ $((i % 5)) -eq 0 ] && [ -f "$TEMP_RAW_OUTPUT" ]; then
			if grep -q "Rate limit exceeded" "$TEMP_RAW_OUTPUT" 2>/dev/null || grep -q '"httpCode":429' "$TEMP_RAW_OUTPUT" 2>/dev/null; then
				RATE_LIMITED_THIS_ATTEMPT=true
			fi
		fi
		sleep 1
	done

	if kill -0 "$CLI_PID" 2>/dev/null; then
		print_info "CLI process timed out after $TIMEOUT seconds, terminating..."
		kill -TERM "$CLI_PID" 2>/dev/null || true
		wait "$CLI_PID" 2>/dev/null || true
		CLI_EXIT_CODE=$?
	fi

	if [ "$CLI_EXIT_CODE" -eq 0 ]; then
		print_status "CLI exited successfully (assumed proof submission)"
		exit 0
	fi

	if [ "$RATE_LIMITED_THIS_ATTEMPT" = true ]; then
		if [ $ATTEMPT_COUNT -lt 2 ]; then
			print_info "Rate limited (attempt $ATTEMPT_COUNT). Retrying with next node id..."
			rm -f "$TEMP_RAW_OUTPUT"
			continue
		else
			print_error "Integration test FAILED - Rate limited across attempts"
			exit 429
		fi
	else
		print_info "CLI output (last 10 lines):"; tail -10 "$TEMP_RAW_OUTPUT" || true
		print_error "Integration test FAILED - Non-zero exit (not rate limited)"
		exit 1
	fi

done

print_error "Integration test FAILED - No proof submission detected"
exit 1 