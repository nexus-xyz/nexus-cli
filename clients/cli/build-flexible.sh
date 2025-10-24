#!/bin/bash

# Flexible CPU Optimization Build Script
# Choose between native (maximum performance) or specific CPU (better portability)

set -e

echo "🚀 Nexus CLI Build Script"
echo "Choose CPU optimization:"
echo "1) native (maximum performance, may not work on older CPUs) [LOCAL DEV]"
echo "2) apple-m4 (good performance, works on M1/M2/M3/M4)"
echo "3) apple-m1 (maximum compatibility, works on all Apple Silicon)"
echo "4) generic (maximum compatibility, works everywhere) [CI-SAFE]"

read -p "Enter choice (1-4): " choice

case $choice in
    1)
        CPU_TARGET="native"
        echo "🎯 Using native CPU optimization (maximum performance)"
        ;;
    2)
        CPU_TARGET="apple-m4"
        echo "🎯 Using Apple M4 optimization (good performance + portability)"
        ;;
    3)
        CPU_TARGET="apple-m1"
        echo "🎯 Using Apple M1 optimization (maximum compatibility)"
        ;;
    4)
        CPU_TARGET="generic"
        echo "🎯 Using generic optimization (CI-safe, maximum compatibility)"
        ;;
    *)
        echo "❌ Invalid choice, using generic as safe default"
        CPU_TARGET="generic"
        ;;
esac

echo "🧹 Cleaning previous builds..."
cargo clean

echo "⚡ Building with $CPU_TARGET CPU optimizations..."
export RUSTFLAGS="-C target-cpu=$CPU_TARGET -C opt-level=3 -C panic=abort"

cargo build --release --features build_proto

echo "✅ Build complete!"
echo "📊 Binary info:"
ls -lh target/release/nexus-network
file target/release/nexus-network

echo "🎯 Performance optimizations applied:"
echo "   • CPU targeting: $CPU_TARGET"
echo "   • Maximum optimization level (3)"
echo "   • Full LTO (Link Time Optimization)"
echo "   • Single codegen unit"
echo "   • Panic abort"
echo "   • Symbol stripping"
