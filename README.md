[![Release](https://img.shields.io/github/v/release/nexus-xyz/nexus-cli.svg)](https://github.com/nexus-xyz/nexus-cli/releases)
[![CI](https://github.com/nexus-xyz/nexus-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/nexus-xyz/nexus-cli/actions)
[![License](https://img.shields.io/badge/License-Apache_2.0-green.svg)](https://github.com/nexus-xyz/nexus-cli/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](https://github.com/nexus-xyz/nexus-cli/blob/main/LICENSE-MIT)

# Nexus CLI

A high-performance command-line interface for contributing proofs to the Nexus network.

## Repository Structure

```txt
nexus-cli/
├── programs/          # Default zkVM programs
│   └── c2pa/         # C2PA image verification program
├── clients/
│   └── cli/          # Main CLI implementation
├── proto/            # Shared network interface definition
└── public/           # Files hosted at cli.nexus.xyz
```

## Programs

The [programs](programs/) directory contains the default zkVM programs that ship with the CLI:

- [C2PA Image Verification](programs/c2pa/README.md) - Verifies image compression with C2PA manifests

## CLI Documentation

For CLI installation and usage instructions, see the sections below.

# Nexus CLI and Programs

This repository contains the Nexus command-line interface (CLI) and a collection of zkVM programs.

## Programs

### [C2PA Image Compression Verification](programs/c2pa/README.md)

A zkVM program that verifies image compression while maintaining C2PA manifest integrity. This program demonstrates:
- Zero-knowledge proof generation for image transformations
- C2PA manifest validation
- Cryptographic verification of content provenance
- Integration with public knowledge repositories (NASA, Wikimedia)

## Development

### Prerequisites

- Rust toolchain (latest stable)
- Git

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test --workspace
```

### Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request

## License

This project is licensed under the same terms as the Nexus zkVM project.

# Nexus Network

[Nexus](https://nexus.xyz/) is a global distributed prover network that unites the world's computers to power a new and
better Internet: the Verifiable Internet.

There have been several testnets so far:

- Testnet 0: [October 8 – 28, 2024](https://blog.nexus.xyz/nexus-launches-worlds-first-open-prover-network/)
- Testnet I: [December 9 – 13, 2024](https://blog.nexus.xyz/the-new-nexus-testnet-is-live/)
- Testnet II: [February 18 – 22, 2025](https://blog.nexus.xyz/testnet-ii-is-open/)

---

## Quick Start

### Installation

#### Precompiled Binary (Recommended)

For the simplest and most reliable installation:

```bash
curl https://cli.nexus.xyz/ | sh
```

This will:
1. Download and install the latest precompiled binary for your platform
2. Prompt you to accept the Terms of Use
3. Start the CLI in interactive mode

### Non-Interactive Installation

For automated installations (e.g., in CI):

```bash
curl -sSf https://cli.nexus.xyz/ -o install.sh
chmod +x install.sh
NONINTERACTIVE=1 ./install.sh
```

---

## Terms of Use

Use of the CLI is subject to the [Terms of Use](https://nexus.xyz/terms-of-use).
First-time users running interactively will be prompted to accept these terms.

---

## Node ID

During the CLI's startup, you'll be asked for your node ID. To skip prompts in a
non-interactive environment, manually create a `~/.nexus/config.json` in the
following format:

```json
{
   "node_id": "<YOUR NODE ID>"
}
```

---

## Current Limitations

- To submit programs to the network for proving, contact
  [growth@nexus.xyz](mailto:growth@nexus.xyz).

---

## Get Help

- [Network FAQ](https://docs.nexus.xyz/layer-1/network-devnet/faq)
- [Discord Community](https://discord.gg/nexus-xyz)
- Technical issues? [Open an issue](https://github.com/nexus-xyz/network-api/issues)

---

## Contributing

Interested in contributing to the Nexus Network CLI? Check out our
[Contributor Guide](./CONTRIBUTING.md) for:

- Development setup instructions
- How to report issues and submit pull requests
- Our code of conduct and community guidelines
- Tips for working with the codebase

For most users, we recommend using the precompiled binaries as described above.
The contributor guide is intended for those who want to modify or improve the CLI
itself.