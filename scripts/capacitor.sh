#!/bin/bash
# Capacitor setup and commands for Rusve

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}Rusve Capacitor Setup${NC}"

# Check if we're in the right directory
if [ ! -f "Makefile" ]; then
    echo -e "${RED}Error: Run this from the project root (rusve/)${NC}"
    exit 1
fi

# @capacitor/cli is installed inside clients/webapp.
# All `cap` commands must run from clients/webapp/ (Capacitor requires a package.json there).
WEBAPP="clients/webapp"
CAP="$WEBAPP/node_modules/.bin/cap"

if [ ! -f "$CAP" ]; then
    echo -e "${RED}Error: @capacitor/cli not found. Run: cd clients/webapp && pnpm install${NC}"
    exit 1
fi

# Run a cap command from inside clients/webapp/
cap_run() {
    (cd "$WEBAPP" && "$OLDPWD/$CAP" "$@")
}

# Install Capacitor dependencies
install_capacitor() {
    echo -e "${YELLOW}Installing Capacitor dependencies...${NC}"
    cd "$WEBAPP"
    pnpm add @capacitor/core @capacitor/cli @capacitor/android @capacitor/ios @capacitor/app @capacitor/preferences
    cd ../..
    echo -e "${GREEN}✓ Capacitor installed${NC}"
}

# Initialize Capacitor (config already exists at clients/webapp/capacitor.config.js)
init_capacitor() {
    echo -e "${YELLOW}Checking Capacitor config...${NC}"
    if [ -f "$WEBAPP/capacitor.config.js" ]; then
        echo -e "${YELLOW}capacitor.config.js already exists, skipping init${NC}"
    else
        echo -e "${RED}capacitor.config.js not found in $WEBAPP/${NC}"
        exit 1
    fi
}

# Add Android platform
add_android() {
    echo -e "${YELLOW}Adding Android platform...${NC}"
    (cd "$WEBAPP" && pnpm run build)
    cap_run add android
    echo -e "${GREEN}✓ Android platform added at clients/android/${NC}"
}

# Add iOS platform
add_ios() {
    echo -e "${YELLOW}Adding iOS platform...${NC}"
    (cd "$WEBAPP" && pnpm run build)
    cap_run add ios
    echo -e "${GREEN}✓ iOS platform added${NC}"
}

# Sync web to native
sync() {
    echo -e "${YELLOW}Syncing web to native...${NC}"
    (cd "$WEBAPP" && pnpm run build:mobile)
    cap_run sync
    echo -e "${GREEN}✓ Sync complete${NC}"
}

# Run on Android
run_android() {
    echo -e "${YELLOW}Running on Android...${NC}"
    cap_run run android
}

# Run on iOS (macOS only)
run_ios() {
    echo -e "${YELLOW}Running on iOS...${NC}"
    cap_run run ios
}

# Open Android Studio
open_android() {
    cap_run open android
}

# Open Xcode (macOS only)
open_ios() {
    cap_run open ios
}

# Full setup
setup() {
    install_capacitor
    add_android
    echo -e "${GREEN}✓ Full setup complete!${NC}"
    echo ""
    echo "Next steps:"
    echo "  - To run on Android: ./scripts/capacitor.sh run:android"
    echo "  - To sync changes:  ./scripts/capacitor.sh sync"
}

# Show usage
usage() {
    echo "Usage: ./scripts/capacitor.sh <command>"
    echo ""
    echo "Commands:"
    echo "  install       Install Capacitor dependencies"
    echo "  add:android   Add Android platform"
    echo "  add:ios       Add iOS platform"
    echo "  sync          Build web and sync to native"
    echo "  run:android   Run on Android device/emulator"
    echo "  run:ios       Run on iOS simulator (macOS only)"
    echo "  open:android  Open Android Studio"
    echo "  open:ios      Open Xcode (macOS only)"
    echo "  setup         Full setup (install + add android)"
}

# Parse command
case "$1" in
    install)
        install_capacitor
        ;;
    init)
        init_capacitor
        ;;
    add:android)
        add_android
        ;;
    add:ios)
        add_ios
        ;;
    sync)
        sync
        ;;
    run:android)
        run_android
        ;;
    run:ios)
        run_ios
        ;;
    open:android)
        open_android
        ;;
    open:ios)
        open_ios
        ;;
    setup)
        setup
        ;;
    *)
        usage
        ;;
esac
