#!/bin/bash

# Test script for the version cache function
# This script tests the caching behavior and header forwarding

CACHE_URL="https://us-central1-nexus-cli.cloudfunctions.net/version"
ORIGIN_URL="https://cli.nexus.xyz/version.json"

echo "🧪 Testing Version Cache Function"
echo "=================================="
echo ""

echo "📡 Testing origin response headers:"
echo "curl -I $ORIGIN_URL"
curl -I "$ORIGIN_URL"
echo ""

echo "📦 Testing cache function (first request - should be MISS):"
echo "curl -I $CACHE_URL"
curl -I "$CACHE_URL"
echo ""

echo "⚡ Testing cache function (second request - should be HIT):"
echo "curl -I $CACHE_URL"
curl -I "$CACHE_URL"
echo ""

echo "📄 Testing response content:"
echo "curl -s $CACHE_URL | jq ."
curl -s "$CACHE_URL" | jq .
echo ""

echo "✅ Test complete!" 