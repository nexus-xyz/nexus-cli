#!/bin/bash

# Master integration test runner (repo root). Runs all CLI integration tests.
# Usage: ./scripts/run_integration_tests.sh

set -e
set -o pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI_DIR="$ROOT_DIR/clients/cli"

YELLOW='\033[1;33m'
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

info()  { echo -e "${YELLOW}[INFO]${NC} $1"; }
pass()  { echo -e "${GREEN}[PASS]${NC} $1"; }
fail()  { echo -e "${RED}[FAIL]${NC} $1"; }

cd "$CLI_DIR"

# Require a prebuilt release binary (from the build job) to avoid rebuilds during CI.
resolve_prebuilt_binary() {
	HOST_TRIPLE=$(rustc -vV | awk '/host/ {print $2}')
	BIN_PATH="target/${HOST_TRIPLE}/release/nexus-network"
	if [ -f "${BIN_PATH}.exe" ]; then
		echo "${BIN_PATH}.exe"
		return 0
	fi
	if [ -f "$BIN_PATH" ]; then
		echo "$BIN_PATH"
		return 0
	fi
	# Also check default target dir (without explicit target), just in case
	if [ -f "target/release/nexus-network" ]; then
		echo "target/release/nexus-network"
		return 0
	fi
	if [ -f "target/release/nexus-network.exe" ]; then
		echo "target/release/nexus-network.exe"
		return 0
	fi
	# No prebuilt binary found
	echo ""
}

run_cli() {
	BIN=$(resolve_prebuilt_binary)
	if [ -z "$BIN" ] || [ ! -x "$BIN" ]; then
		fail "Prebuilt binary not found. Please run the Build step first."
		exit 1
	fi
	RUST_LOG=warn "$BIN" start "$@"
}

run_positive_test() {
	info "Starting positive integration test (prebuilt binary required)..."
	ulimit -c 0 || true

	# Node IDs
	NODE_IDS_ENV="${INTEGRATION_TEST_NODE_IDS}"
	if [ -n "$NODE_IDS_ENV" ]; then
		IFS=',' read -ra NODE_IDS <<<"$NODE_IDS_ENV"
	else
		NODE_IDS=("6166715" "23716208" "23519580" "23718361")
	fi
	if command -v shuf >/dev/null 2>&1; then
		NODE_IDS=($(printf '%s\n' "${NODE_IDS[@]}" | shuf))
	fi

	ATTEMPT_COUNT=0
	for node_id in "${NODE_IDS[@]}"; do
		if [ $ATTEMPT_COUNT -ge 2 ]; then
			break
		fi
		ATTEMPT_COUNT=$((ATTEMPT_COUNT+1))

		if [ -n "$GITHUB_ACTIONS" ] || [ -n "$CI" ]; then
			TEMP_RAW_OUTPUT="$PWD/integration_positive.log"
			rm -f "$TEMP_RAW_OUTPUT" 2>/dev/null || true
		else
			TEMP_RAW_OUTPUT=$(mktemp)
			trap "rm -f $TEMP_RAW_OUTPUT" EXIT
		fi
		info "Running CLI (attempt $ATTEMPT_COUNT) with node $node_id..."

		set +e
		run_cli --headless --max-tasks 1 --node-id $node_id 2>&1 | tee "$TEMP_RAW_OUTPUT"
		CLI_EXIT_CODE=$?
		set -e

		if [ "$CLI_EXIT_CODE" -eq 0 ]; then
			pass "Positive test passed"
			return 0
		fi

		if grep -q "Rate limit exceeded" "$TEMP_RAW_OUTPUT" 2>/dev/null || grep -q '"httpCode":429' "$TEMP_RAW_OUTPUT" 2>/dev/null; then
			if [ $ATTEMPT_COUNT -lt 2 ]; then
				info "Rate limited (attempt $ATTEMPT_COUNT). Retrying with next node id..."
				rm -f "$TEMP_RAW_OUTPUT"
				continue
			else
				fail "Positive test failed - Rate limited across attempts"
				return 429
			fi
		else
			info "CLI output (last 10 lines):"; tail -10 "$TEMP_RAW_OUTPUT" || true
			fail "Positive test failed - Non-zero exit (not rate limited)"
			return 1
		fi
	done

	fail "Positive test failed - No proof submission detected"
	return 1
}

run_negative_test() {
	info "Starting negative integration test (invalid node id)..."
	ulimit -c 0 || true
	INVALID_NODE_ID="0"
	if [ -n "$GITHUB_ACTIONS" ] || [ -n "$CI" ]; then
		TMP_OUT="$PWD/integration_negative.log"
		rm -f "$TMP_OUT" 2>/dev/null || true
	else
		TMP_OUT=$(mktemp)
		trap "rm -f $TMP_OUT" EXIT
	fi

	set +e
	run_cli --headless --max-tasks 1 --node-id "$INVALID_NODE_ID" 2>&1 | tee "$TMP_OUT"
	EXIT_CODE=$?
	set -e

	if [ "$EXIT_CODE" -ne 0 ]; then
		pass "Negative test passed (non-zero exit: $EXIT_CODE)"
		return 0
	else
		fail "Negative test failed (CLI exited 0 with invalid node id)"
		info "Last 20 lines of output:"; tail -20 "$TMP_OUT" || true
		return 1
	fi
}

# Run tests sequentially in foreground
run_positive_test
POSITIVE_CODE=$?
if [ $POSITIVE_CODE -ne 0 ]; then
	exit $POSITIVE_CODE
fi

run_negative_test
NEGATIVE_CODE=$?
if [ $NEGATIVE_CODE -ne 0 ]; then
	exit $NEGATIVE_CODE
fi

info "All integration tests passed"
exit 0 