#!/bin/bash

# Firebase Functions Deployment Script
# This script builds and deploys the version cache function

set -e

echo "🔨 Installing dependencies..."
npm install

echo "🏗️  Building TypeScript..."
npm run build

echo "🚀 Deploying functions..."
firebase deploy --only functions

echo "✅ Deployment complete!"
echo ""
echo "Function URLs:"
echo "  Version Cache: https://us-central1-nexus-cli.cloudfunctions.net/version"
echo ""
echo "You can test the function with:"
echo "  curl https://us-central1-nexus-cli.cloudfunctions.net/version" 