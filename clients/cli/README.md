# 🚀 Nexus CLI

The Nexus CLI lets you run a **prover node** and contribute proofs to the Nexus network.
It is the **highest-performance** way to participate.

---

## 🪄 Quick Start

Run the installation script and start the CLI with your node ID:

```bash
curl https://cli.nexus.xyz/ | sh
nexus-network start --node-id <your-node-id>
```

You may register a user account and obtain a node ID at [app.nexus.xyz](https://app.nexus.xyz), or with:

```bash
nexus-network register-user --wallet-address <your-wallet-address>
nexus-network register-node
```

For troubleshooting or to see available command line options, run:

```bash
nexus-network --help start
```

Or, with Docker:

```bash
docker pull nexusxyz/nexus-cli:latest
docker run -it --init nexusxyz/nexus-cli:latest start --node-id <your-node-id>
```

Or, from source:

```
cd clients/cli
cargo run -r -- start --node-id <your-node-id>
```

You may obtain a Devnet node ID to connect to the Nexus network by following the instructions below:

1) Go to https://app.nexus.xyz/nodes,
2) Sign in,
3) Click on the '+ Add Node' button,
4) Select 'Add CLI node',
5) You will be given a node ID to add to this CLI

To clear credentials and log out of the CLI, run:

```bash
cargo run -r -- logout
```

## 📜 Terms of Use

Use of the CLI is subject to the [Terms of Use](https://nexus.xyz/terms-of-use).
The first time you run it, it prompts you to accept the terms. To accept the terms
noninteractively (for example, in a continuous integration environment),
add `NONINTERACTIVE=1` before `sh`.

## System Requirements

The Nexus CLI requires at least 4 GB of RAM. If multithreading is enabled, allot 2.5GB of RAM per thread.

## ⚠️ Known issues

* Only the latest version of the CLI is currently supported.
* Linking email to prover id is currently available on the web version only.
* Counting cycles proved is not yet available in the CLI.
* Only proving is supported. Submitting programs to the network is in private beta.
  To request an API key, contact us at growth@nexus.xyz.

## 📚 Resources

* [Network FAQ](https://docs.nexus.xyz/layer-1/network-devnet/faq)
* [Discord server](https://discord.gg/nexus-xyz)

## 🛠 Developer Requirements

### Linux

```bash
sudo apt update
sudo apt upgrade
sudo apt install build-essential pkg-config libssl-dev git-all
sudo apt install protobuf-compiler
```

### macOS

```bash
# Install using Homebrew
brew install protobuf

# Verify installation
protoc --version
```

### Windows

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

## Creating a Release

To create a release, update the package version in `Cargo.toml`, then create and push a new (annotated) tag, e.g.:

```bash
git tag -a v0.1.2 -m "Release v0.1.2"
git push origin v0.1.2
```

This will trigger the GitHub Actions release workflow that compiles binaries and pushes the Docker image, in
addition to creating release.

**WARNING**: Creating a release through the GitHub UI creates a new release but does **NOT** trigger
the workflow. This leads to a release without a Docker image or binaries, which breaks the installation script.