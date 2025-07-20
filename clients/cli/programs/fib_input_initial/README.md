# Fibonacci Input Initial Guest Program

This guest program implements a Fibonacci sequence calculator that takes three inputs and computes the nth Fibonacci number starting from custom initial values.

## Overview

The program computes the Fibonacci sequence starting from two custom initial values (`init_a` and `init_b`) and returns the value at position `n`. This allows for more flexible Fibonacci calculations beyond the standard sequence starting with 0, 1.

## Input Parameters

The program takes three public inputs in the following order:

1. **`n`** (u32): The position in the Fibonacci sequence to compute
2. **`init_a`** (u32): The first initial value of the sequence
3. **`init_b`** (u32): The second initial value of the sequence

## Algorithm

The program uses an iterative approach to compute the Fibonacci sequence:

```rust
fn fib_iter(n: u32, init_a: u32, init_b: u32) -> u32 {
    let mut a = init_a;
    let mut b = init_b;

    for i in 0..n + 1 {
        if i > 1 {
            let c = a + b;
            a = b;
            b = c;
        }
    }
    b
}
```

## Examples

- `fib_iter(5, 0, 1)` → Returns the 5th Fibonacci number in the standard sequence (0, 1, 1, 2, 3, 5) = **5**
- `fib_iter(3, 2, 3)` → Returns the 3rd Fibonacci number in the sequence (2, 3, 5, 8) = **8**
- `fib_iter(0, 10, 20)` → Returns the 0th number in the sequence (10, 20) = **20**

## Building

This guest program is built using the Nexus zkVM SDK. For detailed instructions on setting up and using the SDK, see the [Nexus zkVM SDK Quick Start Guide](https://docs.nexus.xyz/zkvm/proving/sdk).

### Prerequisites

1. Install Rust: https://rust-lang.org/tools/install
2. Add RISC-V target: `rustup target add riscv32i-unknown-none-elf`
3. Install Nexus zkVM: `rustup run nightly-2025-04-06 cargo install --git https://github.com/nexus-xyz/nexus-zkvm cargo-nexus --tag 'v0.3.4'`

### Build Commands

```bash
# Build for RISC-V target
cargo build --target riscv32i-unknown-none-elf --release

# The ELF file will be generated at:
# target/riscv32i-unknown-none-elf/release/guest
```

## Integration with CLI

This guest program is integrated into the Nexus CLI for anonymous proving. The CLI automatically:

1. Loads the compiled ELF file
2. Provides the three input parameters
3. Executes the program in the zkVM
4. Generates a zero-knowledge proof of correct execution

## Testing

You can test this program using the CLI's anonymous proving feature:

```bash
cargo run -r start --headless
```

The CLI will automatically run this program with hardcoded test inputs and generate a proof.

## Related Documentation

- [Nexus zkVM SDK Quick Start](https://docs.nexus.xyz/zkvm/proving/sdk) - Complete setup and usage guide
- [Nexus zkVM Architecture](https://docs.nexus.xyz/zkvm/architecture) - Technical details about the zkVM
- [Nexus zkVM Runtime](https://docs.nexus.xyz/zkvm/proving/runtime) - Runtime features and APIs 
