#!/bin/bash

# CI-Aware Build Script for Nexus CLI
# Shows what CPU targeting happens in different CI environments

set -e

echo "🚀 Nexus CLI CI-Aware Build Script"
echo "=================================="

# Detect current platform
PLATFORM=$(rustc --version --verbose | grep "host:" | cut -d' ' -f2)
echo "📋 Detected platform: $PLATFORM"

case $PLATFORM in
    "aarch64-apple-darwin")
        echo "🍎 Apple Silicon Mac detected"
        echo "🎯 CPU target: apple-m4 (optimized for M4)"
        echo "✅ Performance: Excellent"
        echo "✅ Portability: Works on M1/M2/M3/M4"
        ;;
    "x86_64-apple-darwin")
        echo "🍎 Intel Mac detected"
        echo "🎯 CPU target: native (Intel optimizations)"
        echo "⚠️  Note: Intel Mac performance not a concern per requirements"
        ;;
    "x86_64-unknown-linux-gnu")
        echo "🐧 Linux x86_64 detected"
        echo "🎯 CPU target: native (Linux x86_64 optimizations)"
        echo "✅ Performance: Optimized for CI runner CPU"
        ;;
    "aarch64-unknown-linux-gnu")
        echo "🐧 Linux ARM64 detected"
        echo "🎯 CPU target: native (Linux ARM64 optimizations)"
        echo "✅ Performance: Optimized for ARM CI runner"
        ;;
    "x86_64-pc-windows-gnu")
        echo "🪟 Windows/WSL detected"
        echo "🎯 CPU target: native (Windows x86_64 optimizations)"
        echo "✅ Performance: Optimized for Windows CI runner"
        ;;
    *)
        echo "❓ Unknown platform: $PLATFORM"
        echo "🎯 CPU target: native (fallback)"
        ;;
esac

echo ""
echo "🔧 Building with platform-specific optimizations..."

# Clean build
echo "🧹 Cleaning previous builds..."
cargo clean

# Build with platform-specific optimizations
echo "⚡ Building for $PLATFORM..."
cargo build --release --features build_proto

echo ""
echo "✅ Build complete!"
echo "📊 Binary info:"
ls -lh target/release/nexus-network
file target/release/nexus-network

echo ""
echo "🎯 Platform-specific optimizations applied:"
echo "   • Platform: $PLATFORM"
echo "   • CPU targeting: Platform-specific"
echo "   • Maximum optimization level (3)"
echo "   • Full LTO (Link Time Optimization)"
echo "   • Single codegen unit"
echo "   • Panic abort"
echo "   • Symbol stripping"

echo ""
echo "📋 CI Environment Behavior:"
echo "   • GitHub Actions (Linux): Uses native CPU targeting"
echo "   • GitHub Actions (macOS): Uses apple-m4 (ARM) or native (Intel)"
echo "   • WSL: Uses native CPU targeting"
echo "   • Local Apple Silicon: Uses apple-m4 targeting"
echo "   • Local Intel Mac: Uses native targeting (not a concern)"

