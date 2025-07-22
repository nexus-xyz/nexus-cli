[![Release](https://img.shields.io/github/v/release/nexus-xyz/nexus-cli.svg)](https://github.com/nexus-xyz/nexus-cli/releases)
[![CI](https://github.com/nexus-xyz/nexus-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/nexus-xyz/nexus-cli/actions)
[![License](https://img.shields.io/badge/License-Apache_2.0-green.svg)](https://github.com/nexus-xyz/nexus-cli/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](https://github.com/nexus-xyz/nexus-cli/blob/main/LICENSE-MIT)
[![Twitter](https://img.shields.io/twitter/follow/NexusLabs)](https://x.com/NexusLabs)
[![Discord](https://img.shields.io/badge/Discord-Join-7289da.svg?logo=discord&logoColor=white)](https://discord.com/invite/nexus-xyz)

# Nexus CLI

A high-performance command-line interface for contributing proofs to the Nexus network.

<figure>
    <a href="https://nexus.xyz/">
        <img src="assets/images/nexus-network-image.png" alt="Nexus Network visualization showing a distributed network of interconnected nodes with a 'Launch Network' button in the center">
    </a>
    <figcaption>
        <strong>Verifiable Computation on a Global Scale</strong><br>
        We're building a global distributed prover network to unite the world's computers and power a new and better Internet: the Verifiable Internet. Connect to the beta and give it a try today.
    </figcaption>
</figure>

## Nexus Network

[Nexus](https://nexus.xyz/) is a global distributed prover network that unites the world's computers to power a new and
better Internet: the Verifiable Internet.

There have been several testnets so far:

- Testnet 0: [October 8 – 28, 2024](https://blog.nexus.xyz/nexus-launches-worlds-first-open-prover-network/)
- Testnet I: [December 9 – 13, 2024](https://blog.nexus.xyz/the-new-nexus-testnet-is-live/)
- Testnet II: [February 18 – 22, 2025](https://blog.nexus.xyz/testnet-ii-is-open/)
- Devnet: [February 22 - June 20 2025](https://docs.nexus.xyz/layer-1/testnet/devnet)
- Testnet III: [Ongoing](https://blog.nexus.xyz/live-everywhere/)

---

## Quick Start

### Installation

#### Precompiled Binary (Recommended)

For the simplest and most reliable installation:

```bash
curl https://cli.nexus.xyz/ | sh
```

This will:
1. Download and install the latest precompiled binary for your platform.
2. Prompt you to accept the Terms of Use.
3. Start the CLI in interactive mode.

The exact installation script is viewable [here](./public/install.sh).

#### Non-Interactive Installation

For automated installations (e.g., in CI):

```bash
curl -sSf https://cli.nexus.xyz/ -o install.sh
chmod +x install.sh
NONINTERACTIVE=1 ./install.sh
```

### Proving

Proving with the CLI is documented [here](https://docs.nexus.xyz/layer-1/testnet/cli-node).

To start with an existing node ID, run:

```bash
nexus-cli start --node-id <your-node-id>
```

Alternatively, you can register your wallet address and create a node ID with the CLI, or at [app.nexus.xyz](https://app.nexus.xyz).

```bash
nexus-cli register-user --wallet-address <your-wallet-address>
nexus-cli register-node
nexus-cli start
```

The `register-user` and `register-node` commands will save your credentials to `~/.nexus/config.json`. To clear credentials, run:

```bash
nexus-cli logout
```

For troubleshooting or to see available command line options, run:

```bash
nexus-cli --help
```

### Use Docker
Make sure docker and docker compose have been installed on you machine. check documentation here:
- [Install Docker](https://docs.docker.com/engine/install/)
- [Install Docker Compose](https://docs.docker.com/compose/install/)

Then, modify the node id in the `docker-compose.yaml` file, run:

```bash
docker compose build --no-cache
docker compose up -d
```

Check log

```bash
docker compose logs
```

If you want to shut down, run:

```bash
docker compose down
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

## Get Help

- [Network FAQ](https://docs.nexus.xyz/layer-1/testnet/faq)
- [Discord Community](https://discord.gg/nexus-xyz)
- Technical issues? [Open an issue](https://github.com/nexus-xyz/nexus-cli/issues)
- To submit programs to the network for proving, contact
  [growth@nexus.xyz](mailto:growth@nexus.xyz).

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

### 🛠  Developer Guide

The following steps may be required in order to setup a development environment for contributing to the project:

#### Linux

```bash
sudo apt update
sudo apt upgrade
sudo apt install build-essential pkg-config libssl-dev git-all
sudo apt install protobuf-compiler
```

#### macOS

```bash
# Install using Homebrew
brew install protobuf

# Verify installation
protoc --version
```

#### Windows

[Install WSL](https://learn.microsoft.com/en-us/windows/wsl/install),
then see Linux instructions above.

```bash
# Install using Chocolatey
choco install protobuf
```

### Building ProtoBuf files

To build the ProtoBuf files, run the following command in the `clients/cli` directory:

```bash
cargo build --features build_proto
```

### Creating a Release

To create a release, update the package version in `Cargo.toml`, then create and push a new (annotated) tag, e.g.:

```bash
git tag -a v0.1.2 -m "Release v0.1.2"
git push origin v0.1.2
```

This will trigger the GitHub Actions release workflow that compiles binaries and pushes the Docker image, in
addition to creating release.

**WARNING**: Creating a release through the GitHub UI creates a new release but does **NOT** trigger
the workflow. This leads to a release without a Docker image or binaries, which breaks the installation script.
