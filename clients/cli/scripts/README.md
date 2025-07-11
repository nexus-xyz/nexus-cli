# CLI Scripts

This directory contains utility scripts for the Nexus CLI.

## Build Guest Script

The `build_guest.sh` script automates the process of creating and building guest programs using `cargo nexus host`.

### Usage

```bash
./scripts/build_guest.sh [guest_name]
```

### Parameters

- `guest_name` (optional): Name for the guest program. Defaults to `fib_input_initial`

### Examples

```bash
# Build with default name
./scripts/build_guest.sh

# Build with custom name
./scripts/build_guest.sh my_custom_program
```

### What it does

1. Creates a `programs/` directory (if it doesn't exist)
2. Uses `cargo nexus host` to generate a new guest program (or rebuilds existing one)
3. Builds the guest program for RISC-V target (`riscv32i-unknown-none-elf`)
4. Copies the built ELF to `assets/[guest_name]` in the CLI repo
5. Provides colored output and error handling

### Directory Structure

After running the script, you'll have:

```
nexus-cli/clients/cli/
├── programs/                 # Guest program source code (tracked in git)
│   └── [guest_name]/         # Generated guest program
│       ├── src/
│       │   └── guest/
│       │       └── src/
│       │           └── main.rs  # Edit this file to modify the guest program
│       └── ...
├── assets/
│   └── [guest_name]          # Built ELF file
└── scripts/
    └── build_guest.sh        # This script
```

### Modifying Guest Programs

To modify a guest program:

1. Edit the source code: `programs/[guest_name]/src/guest/src/main.rs`
2. Run the build script again: `./scripts/build_guest.sh [guest_name]`
3. The updated ELF will be copied to `assets/[guest_name]`
4. Commit your source code changes to git

### Git Integration

- **Source code is tracked**: The guest program source code in `programs/` is committed to git
- **Build artifacts are ignored**: `target/` directories and `Cargo.lock` files are excluded
- **Reproducible builds**: Anyone can clone the repo and rebuild the ELF files

### Requirements

- `cargo nexus` must be installed and available in PATH
- The CLI assets directory must exist at `assets/`

### Error Handling

The script will exit with an error if:
- `cargo nexus` is not available
- The build fails
- The ELF file can't be copied

All errors are displayed with red text and include helpful error messages. 