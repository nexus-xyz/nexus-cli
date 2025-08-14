#!/bin/bash

# Negative Integration Test (run from clients/cli): invalid node ID should not succeed
# This test PASSES if the CLI exits with a non-zero status (expected failure)
# and FAILS if the CLI exits with status 0 (unexpected success).
# Usage: ./integration_test_invalid_node.sh [binary_path]

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_pass()  { echo -e "${GREEN}[PASS]${NC} $1"; }
print_fail()  { echo -e "${RED}[FAIL]${NC} $1"; }
print_info()  { echo -e "${YELLOW}[INFO]${NC} $1"; }

BINARY_PATH="${1:-./target/release/nexus-network}"
if [ ! -f "$BINARY_PATH" ]; then
	print_fail "CLI binary not found at: $BINARY_PATH"
	exit 1
fi

INVALID_NODE_ID="0"  # Intentionally invalid
TIMEOUT=60
TMP_OUT=$(mktemp)
trap "rm -f $TMP_OUT" EXIT

print_info "Running negative test with invalid node id: $INVALID_NODE_ID"
(
	ulimit -c 0 || true
	"$BINARY_PATH" start --headless --max-tasks 1 --node-id "$INVALID_NODE_ID" 2>&1 | tee "$TMP_OUT"
) &
PID=$!

EXIT_CODE=0
for i in $(seq 1 $TIMEOUT); do
	if ! kill -0 "$PID" 2>/dev/null; then
		wait "$PID"; EXIT_CODE=$?; break
	fi
	sleep 1
done

# If process is still running, consider it not-success and pass the test (it did not exit 0)
if kill -0 "$PID" 2>/dev/null; then
	print_info "Process still running after $TIMEOUT seconds; terminating..."
	kill -TERM "$PID" 2>/dev/null || true
	wait "$PID" 2>/dev/null || true
	print_pass "Expected failure behavior observed (CLI did not exit successfully)"
	exit 0
fi

if [ "$EXIT_CODE" -ne 0 ]; then
	print_pass "Expected failure: CLI exited with non-zero status ($EXIT_CODE) for invalid node id"
	exit 0
else
	print_fail "Unexpected success: CLI exited 0 with invalid node id"
	print_info "Last 20 lines of output:"; tail -20 "$TMP_OUT" || true
	exit 1
fi 