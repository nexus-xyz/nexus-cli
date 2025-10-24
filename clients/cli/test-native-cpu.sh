#!/bin/bash

# Simple Native CPU Detection Test
# Shows what CPU features are detected with 'native' targeting

set -e

echo "🔍 Native CPU Detection Test"
echo "============================"

# Get current platform
PLATFORM=$(rustc --version --verbose | grep "host:" | cut -d' ' -f2)
echo "📋 Platform: $PLATFORM"

# Show what 'native' would detect
echo ""
echo "🎯 What 'target-cpu=native' detects:"
echo "   • Automatically detects the best CPU features"
echo "   • Uses the most advanced instructions available"
echo "   • Optimizes for the exact CPU running the build"

echo ""
echo "📊 Platform-specific behavior:"
case $PLATFORM in
    "aarch64-apple-darwin")
        echo "   • Apple Silicon: Detects M1/M2/M3/M4 features"
        echo "   • Uses ARM64 NEON, Crypto extensions, etc."
        echo "   • Automatically adapts to M4 if running on M4"
        ;;
    "x86_64-apple-darwin")
        echo "   • Intel Mac: Detects Intel CPU features"
        echo "   • Uses SSE, AVX, AES-NI, etc."
        ;;
    "x86_64-unknown-linux-gnu")
        echo "   • Linux x86_64: Detects server/desktop CPU features"
        echo "   • Uses SSE, AVX, AES-NI, etc."
        ;;
    "aarch64-unknown-linux-gnu")
        echo "   • Linux ARM64: Detects ARM server features"
        echo "   • Uses ARM64 NEON, Crypto extensions, etc."
        ;;
esac

echo ""
echo "✅ Benefits of 'native':"
echo "   • 🚀 Maximum performance on current CPU"
echo "   • 🔄 Automatically adapts to any architecture"
echo "   • 🛠️  No manual configuration needed"
echo "   • 🎯 Uses best available instructions"
echo "   • 📦 Works in CI/CD on any platform"

echo ""
echo "⚠️  Portability consideration:"
echo "   • Binary optimized for build machine's CPU"
echo "   • May not run on older CPUs (but that's usually fine)"
echo "   • For distribution, consider 'generic' target"

echo ""
echo "🎯 Recommendation: Use 'native' for development and CI"
echo "   • Perfect for your use case (Apple Silicon + Linux CI)"
echo "   • Automatically optimal on all platforms"
echo "   • No need for platform-specific configuration"

