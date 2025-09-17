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
- Devnet: [February 22 - June 20, 2025](https://docs.nexus.xyz/layer-1/testnet/devnet)
- Testnet III: [Ongoing](https://blog.nexus.xyz/live-everywhere/)

---

## Quick Start

### Installation

#### Precompiled Binary (Recommended)

For the simplest and most reliable installation:

```bash
curl https://cli.nexus.xyz/ | sh
```

This downloads the latest binary, prompts for Terms of Use acceptance, and starts interactive mode.

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
nexus-cli register-node --node-id <your-cli-node-id>
nexus-cli start
```

To run the CLI noninteractively, you can also opt to start it in headless mode.

```bash
nexus-cli start --headless
```

#### Quick Reference

The `register-user` and `register-node` commands will save your credentials to `~/.nexus/config.json`. To clear credentials, run:

```bash
nexus-cli logout
```

For troubleshooting or to see available command-line options, run:

```bash
nexus-cli --help
```

### Adaptive Task Difficulty

The Nexus CLI features an **adaptive difficulty system** that automatically adjusts task difficulty based on your node's performance. This ensures optimal resource utilization while preventing system overload.

#### How It Works

- **Starts at**: `small_medium` difficulty
- **Auto-promotes**: `small_medium` → `medium` → `large` → `extra_large` → `extra_large_2` (if tasks complete in < 7 minutes)
- **Safety**: `small` difficulty does not auto-promote (manual override only)

```
small_medium → medium → large → extra_large → extra_large_2
     ↑           ↑        ↑         ↑            ↑
   Default    < 7 min   < 7 min   < 7 min     < 7 min
              success   success   success     success
```

#### When to Override Difficulty

**Lower Difficulty** (`Small` or `SmallMedium`):
- Resource-constrained systems
- Background processing alongside other apps
- Testing/development environments
- Battery-powered devices

**Higher Difficulty** (`Large`, `ExtraLarge`, or `ExtraLarge2`):
- High-performance hardware (8+ cores, 16+ GB RAM)
- Dedicated proving machines
- Experienced users who understand requirements
- Maximum reward optimization

#### Using Difficulty Override

```bash
# Lower difficulty for resource-constrained systems
nexus-cli start --max-difficulty small
nexus-cli start --max-difficulty small_medium

# Higher difficulty for powerful hardware
nexus-cli start --max-difficulty large
nexus-cli start --max-difficulty extra_large
nexus-cli start --max-difficulty extra_large_2

# Case-insensitive (all equivalent)
nexus-cli start --max-difficulty MEDIUM
nexus-cli start --max-difficulty medium
nexus-cli start --max-difficulty Medium
```

**Available Difficulty Levels:**
- `small` - Basic tasks, minimal resource usage
- `small_medium` - Default starting difficulty, balanced performance
- `medium` - Moderate complexity, good for most systems
- `large` - High complexity, requires powerful hardware
- `extra_large` - Very high complexity, for dedicated high-end systems
- `extra_large_2` - Maximum complexity, for the most powerful systems

#### Difficulty Guidelines

| Difficulty | Use Case |
|------------|----------|
| `small` | Resource-constrained, background processing |
| `small_medium` | Default, balanced performance |
| `medium` | Standard desktop/laptop |
| `large` | High-performance systems |
| `extra_large` | Dedicated proving machines |
| `extra_large_2` | Maximum performance systems |

> **Tip**: Use `nexus-cli start --help` to see the full auto-promotion details in the CLI help text.

#### Monitoring Performance

Monitor performance in the dashboard:
- **Task Completion Time**: Shown in metrics panel
- **Promotion Status**: Whether system is promoting or maintaining level

#### Troubleshooting Difficulty Issues

**Tasks taking too long:**
```bash
nexus-cli start --max-difficulty small_medium
```

**Want more challenging tasks:**
```bash
nexus-cli start --max-difficulty large
```

**Unsure about system capabilities:**
- Use the default adaptive system (no `--max-difficulty` needed)
- The system will automatically find the optimal difficulty for your hardware
- Only override if you're specifically unhappy with the automatic performance

### Docker Installation

For containerized deployments:

1. Install [Docker](https://docs.docker.com/engine/install/) and [Docker Compose](https://docs.docker.com/compose/install/)
2. Update the node ID in `docker-compose.yaml`
3. Build and run:

```bash
docker compose build --no-cache
docker compose up -d
docker compose logs  # Check logs
docker compose down  # Shutdown
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

The following steps may be required in order to set up a development environment for contributing to the project:

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

## License

Nexus CLI is distributed under the terms of both the [MIT License](./LICENSE-MIT) and the [Apache License (Version 2.0)](./LICENSE-APACHE).
