#!/bin/bash

# High-Performance Local Development Build Script for Nexus CLI
# ⚠️  WARNING: Uses native CPU optimizations - NOT for CI/precompiled builds!
# For CI builds, use: ./build-ci-safe.sh

set -e

echo "🚀 Building Nexus CLI with LOCAL DEVELOPMENT optimizations..."
echo "⚠️  Warning: This build may not work on different CPUs!"

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Set environment variables for maximum performance
export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C panic=abort -C lto=fat -C codegen-units=1"

# Build with maximum optimizations
echo "⚡ Building with native CPU optimizations..."
cargo build --release --features build_proto

echo "✅ Build complete!"
echo "📊 Binary size:"
ls -lh target/release/nexus-network

echo "🎯 LOCAL DEVELOPMENT optimizations applied:"
echo "   • Native CPU targeting (maximum performance)"
echo "   • Maximum optimization level (3)"
echo "   • Full LTO (Link Time Optimization)"
echo "   • Single codegen unit"
echo "   • Panic abort (smaller binary)"
echo "   • Symbol stripping"
echo "   • No debug symbols"
echo "   • No incremental compilation"
echo ""
echo "⚠️  IMPORTANT: This binary is optimized for THIS CPU only!"
echo "🏗️  For CI/precompiled builds, use: ./build-ci-safe.sh"
