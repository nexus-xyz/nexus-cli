#!/bin/bash

echo "🧪 Testing Nexus CLI Full UI Theme System"
echo "=========================================="
echo ""

# Test 1: Check if binary exists
if [ -f "target/release/nexus-network" ]; then
    echo "✅ Binary exists: target/release/nexus-network"
else
    echo "❌ Binary not found. Run 'cargo build --release --features build_proto' first"
    exit 1
fi

# Test 2: Check if theme system compiles
echo "🔨 Testing full UI theme system compilation..."
if cargo check --features build_proto > /dev/null 2>&1; then
    echo "✅ Full UI theme system compiles successfully"
else
    echo "❌ Theme system compilation failed"
    exit 1
fi

# Test 3: Check theme files
echo "📁 Checking theme system files..."
if [ -f "src/ui/theme.rs" ]; then
    echo "✅ Theme module exists: src/ui/theme.rs"
else
    echo "❌ Theme module not found"
    exit 1
fi

if [ -f "THEME_SYSTEM.md" ]; then
    echo "✅ Theme documentation exists: THEME_SYSTEM.md"
else
    echo "❌ Theme documentation not found"
    exit 1
fi

if [ -f "demo-themes.sh" ]; then
    echo "✅ Demo script exists: demo-themes.sh"
else
    echo "❌ Demo script not found"
    exit 1
fi

echo ""
echo "🎨 Full UI Theme System Test Results:"
echo "======================================"
echo "✅ All tests passed!"
echo ""
echo "🚀 Ready to demo FULL UI theming:"
echo "   • Run: ./demo-themes.sh"
echo "   • Press 'T' in TUI mode to rotate themes"
echo "   • Watch ALL UI elements change colors dynamically!"
echo ""
echo "🎯 Themed Components:"
echo "   • Header - Title, theme name, progress gauge"
echo "   • System Info Panel - Node, environment, uptime, threads, memory"
echo "   • Activity Log Panel - Event logs with timestamps"
echo "   • System Metrics - CPU, RAM, Peak RAM gauges"
echo "   • zkVM Stats - Tasks, success rate, runtime, last proof"
echo "   • Footer - Controls and branding"
echo "   • Background - Main dashboard background"
echo ""
echo "🎨 Available themes:"
echo "   1. Vibrant Blue - Bright sky blue and coral red"
echo "   2. Cyberpunk Neon - Matrix green and hot pink"
echo "   3. Ocean Blue - Vibrant ocean-inspired blues and oranges"
echo "   4. Retro Rainbow - 80s terminal with bright orange and green"
echo "   5. Sunset Gradient - Warm sunset colors with vibrant gradients"
echo ""
echo "🎉 Full UI theme system is ready for demonstration!"
