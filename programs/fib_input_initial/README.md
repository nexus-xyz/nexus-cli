# Fibonacci Input Initial Program

This directory contains the source code for the `fib_input_initial` guest program used in Nexus CLI for anonymous proving.

## Overview

The `fib_input_initial` program computes the nth Fibonacci number where n is provided as a public input. This program is used as the default program for anonymous proving in the Nexus CLI.

## Building

To build the ELF file:

1. Ensure you have the correct Rust toolchain installed:
   ```bash
   rustup target add riscv32im-unknown-none-elf
   ```

2. Build the program:
   ```bash
   cargo build --release --target riscv32im-unknown-none-elf
   ```

3. The resulting ELF file will be in `target/riscv32im-unknown-none-elf/release/guest`

## Usage

The built ELF file is included in `../clients/cli/assets/fib_input_initial.elf` and is used by the Nexus CLI for anonymous proving operations.

## Program Details

- **Input**: Public input n (u32) representing which Fibonacci number to compute
- **Output**: The nth Fibonacci number
- **Exit Code**: 0 on success, non-zero on error

## Testing

To test the program locally:

```bash
cargo test --target riscv32im-unknown-none-elf
``` 