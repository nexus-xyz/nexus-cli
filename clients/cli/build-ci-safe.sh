#!/bin/bash

# CI-Safe Build Script for Nexus CLI
# Uses safe CPU targeting for maximum compatibility in precompiled builds

set -e

echo "🚀 Building Nexus CLI with CI-safe optimizations..."
echo "🎯 Target: Platform-appropriate safe CPU targeting"

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build with safe CPU targeting for maximum compatibility
# Override the native CPU targeting from .cargo/config.toml
# Note: Using apple-m1 on Apple Silicon for ring crate compatibility
# This still provides good compatibility across M1/M2/M3/M4
PLATFORM=$(rustc --version --verbose | grep "host:" | cut -d' ' -f2)

if [[ "$PLATFORM" == "aarch64-apple-darwin" ]]; then
    echo "⚡ Building with Apple M1 optimizations (CI-safe for Apple Silicon)..."
    echo "   Overriding config.toml target-cpu=native with apple-m1"
    export RUSTFLAGS="-C target-cpu=apple-m1"
else
    echo "⚡ Building with generic CPU optimizations (CI-safe)..."
    echo "   Overriding config.toml target-cpu=native with generic"
    export RUSTFLAGS="-C target-cpu=generic"
fi

cargo build --release --features build_proto

echo ""
echo "✅ CI-Safe build complete!"
echo "📊 Binary info:"
ls -lh target/release/nexus-network
file target/release/nexus-network

echo ""
if [[ "$PLATFORM" == "aarch64-apple-darwin" ]]; then
    echo "🎯 CI-Safe (Apple Silicon) optimizations applied:"
    echo "   • CPU targeting: apple-m1 (compatible with M1/M2/M3/M4)"
else
    echo "🎯 CI-Safe optimizations applied:"
    echo "   • CPU targeting: generic (maximum compatibility)"
fi

echo "   • Maximum optimization level (3)"
echo "   • Full LTO (Link Time Optimization)"
echo "   • Single codegen unit"
echo "   • Panic abort"
echo "   • Symbol stripping"
echo ""
echo "✅ This build will work on target platforms"
echo "🚀 Perfect for CI/CD and precompiled distributions"
