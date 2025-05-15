# C2PA zkVM Program

This directory contains a zkVM program for verifying C2PA signatures.

## Structure

- `c2pa/` - All C2PA-related code
  - `core/` - Core library with verification logic and tests
  - `guest/` - The zkVM guest program

## Building

To build the guest program for the zkVM:

```bash
# From the programs directory
cargo +nightly build -p c2pa-guest --target riscv32im-unknown-none-elf -Z build-std=core,alloc --profile ci-build
```

## Testing

The core library can be tested in a host environment:

```bash
# From the programs/c2pa/core directory
cargo test --features host
```

## Development

- The guest program (`c2pa/guest/`) should only use zkVM precompiles
- Core logic should be implemented in `c2pa/core/` and tested there
- The guest program should be minimal and focused on zkVM integration

## CI

The GitHub Actions workflow:
1. Builds the guest program for zkVM
2. Runs tests on the core library
3. Verifies no-std compatibility

To run the CI checks locally:

```bash
# Install the RISC-V target
rustup +nightly target add riscv32im-unknown-none-elf

# Build the guest program
cd programs
cargo +nightly build -p c2pa-guest --target riscv32im-unknown-none-elf -Z build-std=core,alloc --profile ci-build

# Run core library tests
cd c2pa/core
cargo test --features host

# Verify no-std compatibility
cargo check --target riscv32im-unknown-none-elf -Z build-std=core,alloc
``` 