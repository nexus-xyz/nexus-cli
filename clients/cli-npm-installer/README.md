# Nexus Network CLI Installer

The Nexus Network CLI enables interaction with the Nexus network directly from your terminal. This installer simplifies
setup by automatically downloading and configuring the latest binary for your operating system.

---

## Installation

You have two simple ways to install the Nexus CLI:

### Method 1: Using `curl` (recommended)

```bash
curl https://cli.nexus.xyz/ | sh
```

### Method 2: Using `npm`

If you prefer managing packages with npm, run:

```bash
npm install -g @nexusxyz/nexus-network
```

---

## Supported Platforms

* Linux (x86\_64, arm64)
* macOS (Intel x86\_64, Apple Silicon arm64)
* Windows (x86\_64)

If your platform or architecture isn't supported, the installer will prompt you to build from source.

---

## Usage

After installation, run the Nexus CLI with:

```bash
nexus-network start --env beta
```

This will start the Nexus Network in beta mode, ready to interact with the Nexus Devnet.

---

## Terms of Use

Before installation, you'll be prompted to agree to the [Nexus Beta Terms of Use](https://nexus.xyz/terms-of-use).

---

## Uninstallation

To uninstall, simply remove the binary:

```bash
rm -rf ~/.nexus
rm ~/.local/bin/nexus-network
```

If you installed via npm:

```bash
npm uninstall -g @nexusxyz/nexus-network
```

---

## License

This installer script is provided under the [MIT License](LICENSE).

---

## Support

For support or questions, visit the [Nexus GitHub repository](https://github.com/nexus-xyz/nexus-cli).
