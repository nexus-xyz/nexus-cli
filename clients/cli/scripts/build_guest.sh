#!/bin/bash

# Build script for guest programs
# This script creates a guest program using cargo nexus host and builds it

set -e  # Exit on any error

# Configuration
GUEST_NAME="${1:-fib_input_initial}"
CLI_ASSETS_PATH="$(pwd)/assets"
PROGRAMS_PATH="$(pwd)/programs"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building guest program: ${GUEST_NAME}${NC}"
echo -e "${YELLOW}Programs path: ${PROGRAMS_PATH}${NC}"
echo -e "${YELLOW}CLI assets path: ${CLI_ASSETS_PATH}${NC}"

# Create programs directory if it doesn't exist
mkdir -p "${PROGRAMS_PATH}"

# Navigate to programs directory
cd "${PROGRAMS_PATH}"

# Check if the guest program already exists
if [ -d "${GUEST_NAME}" ]; then
    echo -e "${YELLOW}Guest program ${GUEST_NAME} already exists, rebuilding...${NC}"
    cd "${GUEST_NAME}"
else
    echo -e "${GREEN}Creating new guest program with cargo nexus host...${NC}"
    cargo nexus host "${GUEST_NAME}"
    cd "${GUEST_NAME}"
fi

echo -e "${GREEN}Building guest program...${NC}"
cd src/guest
cargo build --release --target riscv32i-unknown-none-elf

# Check if build was successful
if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Build failed${NC}"
    exit 1
fi

# Copy the built ELF to CLI assets
echo -e "${GREEN}Copying ELF to CLI assets...${NC}"
cp "../../target/riscv32i-unknown-none-elf/release/guest" "${CLI_ASSETS_PATH}/${GUEST_NAME}"

# Check if copy was successful
if [ $? -eq 0 ]; then
    echo -e "${GREEN}Successfully built and copied ${GUEST_NAME} to CLI assets${NC}"
    echo -e "${YELLOW}ELF file: ${CLI_ASSETS_PATH}/${GUEST_NAME}${NC}"
    echo -e "${YELLOW}Guest source: ${PROGRAMS_PATH}/${GUEST_NAME}/src/guest/src/main.rs${NC}"
else
    echo -e "${RED}Error: Failed to copy ELF file${NC}"
    exit 1
fi 