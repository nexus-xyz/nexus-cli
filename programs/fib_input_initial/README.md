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

## Building and Copying the ELF File

To build the guest program and copy the resulting ELF file to the CLI assets directory, run:

```sh
./build_and_copy.sh
```

This script will:
- Build the guest program for the RISC-V target
- Copy the resulting ELF file to `../../clients/cli/assets/fib_input_initial` (no extension)

If the build fails, the script will print an error message.

## Usage

The built ELF file is included in `