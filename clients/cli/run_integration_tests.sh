#!/bin/bash

# Wrapper script to run all CLI integration tests (from clients/cli)
# - Positive test (expected success): ./integration_test.sh
# - Negative test (expected failure with invalid node id): ./integration_test_invalid_node.sh
# Usage: ./run_integration_tests.sh

set -e

YELLOW='\033[1;33m'
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

info()  { echo -e "${YELLOW}[INFO]${NC} $1"; }
pass()  { echo -e "${GREEN}[PASS]${NC} $1"; }
fail()  { echo -e "${RED}[FAIL]${NC} $1"; }

# Run positive test first
info "Running positive integration test..."
./integration_test.sh
POSITIVE_CODE=$?
if [ $POSITIVE_CODE -ne 0 ]; then
	if [ $POSITIVE_CODE -eq 429 ]; then
		fail "Positive test failed due to rate limiting (exit 429)"
		exit 429
	fi
	fail "Positive test failed (exit $POSITIVE_CODE)"
	exit $POSITIVE_CODE
fi
pass "Positive integration test passed"

# Run negative test next (invalid node id)
info "Running negative integration test (invalid node id)..."
./integration_test_invalid_node.sh
NEGATIVE_CODE=$?
if [ $NEGATIVE_CODE -ne 0 ]; then
	fail "Negative integration test failed (exit $NEGATIVE_CODE)"
	exit $NEGATIVE_CODE
fi
pass "Negative integration test passed"

info "All integration tests passed"
exit 0 