# Fibonacci Input Initial Program

This directory contains the source code for the `fib_input_initial` guest program used in Nexus CLI for anonymous proving.

## Overview

The `fib_input_initial` program computes the nth Fibonacci number in a generalized Fibonacci sequence with custom initial values. This program is used as the default program for anonymous proving in the Nexus CLI.

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

- **Input**: Three u32 values in little-endian format:
  - **n** (required): Number of iterations for the Fibonacci sequence
  - **init_a** (optional, defaults to 1): First initial value for the sequence
  - **init_b** (optional, defaults to 1): Second initial value for the sequence
- **Output**: The nth Fibonacci number in the sequence starting with init_a, init_b
- **Exit Code**: 0 on success, non-zero on error

## Examples

- Input `[5, 1, 1]` → Classic Fibonacci: 1, 1, 2, 3, 5, 8 (returns 8)
- Input `[5, 2, 3]` → Custom sequence: 2, 3, 5, 8, 13, 21 (returns 21)
- Input `[3, 0, 1]` → Sequence: 0, 1, 1, 2 (returns 2)

## Testing

To test the program locally:

```bash
cargo test --target riscv32im-unknown-none-elf
``` 