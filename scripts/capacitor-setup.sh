#!/bin/bash
# Add Capacitor dependencies and scripts to webapp/package.json

set -e

PKG_JSON="clients/webapp/package.json"

echo "Adding Capacitor dependencies and scripts to package.json..."

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "Installing jq..."
    sudo apt install -y jq
fi

# Add dependencies
cd clients/webapp && pnpm add @capacitor/core @capacitor/cli @capacitor/android @capacitor/ios && cd ../..

# Add scripts using jq
jq '.scripts += {
    "cap:sync": "npx cap sync --config ../../capacitor.config.js",
    "cap:android": "npx cap open android --config ../../capacitor.config.js",
    "cap:ios": "npx cap open ios --config ../../capacitor.config.js",
    "cap:run:android": "npx cap run android --config ../../capacitor.config.js",
    "cap:run:ios": "npx cap run ios --config ../../capacitor.config.js"
}' "$PKG_JSON" > temp.json && mv temp.json "$PKG_JSON"

echo "✓ Capacitor dependencies and scripts added"
