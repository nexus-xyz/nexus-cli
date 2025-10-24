#!/bin/bash

echo "🎨 Nexus CLI Full UI Theme Demo"
echo "================================"
echo ""
echo "This demo showcases the complete theme system affecting the ENTIRE UI:"
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
echo "🎨 Available Themes:"
echo "   1. Vibrant Blue - Bright sky blue and coral red"
echo "   2. Cyberpunk Neon - Matrix green and hot pink"
echo "   3. Ocean Blue - Vibrant ocean-inspired blues and oranges"
echo "   4. Retro Rainbow - 80s terminal with bright orange and green"
echo "   5. Sunset Gradient - Warm sunset colors with vibrant gradients"
echo ""
echo "🚀 Starting CLI in TUI mode..."
echo "   • Press 'T' to rotate through themes"
echo "   • Press 'Q' or 'Esc' to exit"
echo "   • Watch ALL UI elements change colors dynamically!"
echo ""
echo "Starting in 3 seconds..."
sleep 3

# Start the CLI in TUI mode
cargo run --release --features build_proto -- start --orchestrator-url 'https://staging.orchestrator.nexus.xyz' --node-id 4170008 --max-difficulty small_medium
