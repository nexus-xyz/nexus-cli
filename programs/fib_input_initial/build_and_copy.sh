#!/bin/bash
set -e

# Build the guest program for RISC-V target
echo "Building guest program..."
cargo build --release --target riscv32im-unknown-none-elf

# Path to the built ELF file
ELF_PATH="target/riscv32im-unknown-none-elf/release/guest"
DEST_PATH="../../clients/cli/assets/fib_input_initial"

if [ -f "$ELF_PATH" ]; then
    echo "Copying ELF to $DEST_PATH"
    cp "$ELF_PATH" "$DEST_PATH"
    echo "Done."
else
    echo "Build failed or ELF not found: $ELF_PATH"
    exit 1
fi 