# DoubleZero Setup Guide

DoubleZero provides the networking layer for the Arkhe OS federation, utilizing GRE tunneling and BGP routing to ensure peer-to-peer connectivity across different infrastructures.

## Prerequisites
- Internet connectivity with a public IP address (no NAT)
- x86_64 server
- Supported OS: Ubuntu 22.04+, Debian 11+, or Rocky Linux / RHEL 8+
- Root or sudo privileges

## Quick Installation
Arkhe OS includes a script to automate the setup:
```bash
./scripts/setup_doublezero.sh <testnet|mainnet-beta>
```

## Identity
Each node must have a unique DoubleZero Identity. This is generated during setup using:
```bash
doublezero keygen
```
Your address can be retrieved with:
```bash
doublezero address
```

## Network Configuration
DoubleZero uses:
- **GRE tunneling**: IP protocol 47
- **BGP routing**: tcp/179 on link-local addresses

Ensure your firewall (iptables/UFW) allows these protocols on the `doublezero0` interface.

## Monitoring
Prometheus metrics can be enabled to monitor client performance and latency:
```bash
# Example override.conf for systemd
[Service]
ExecStart=
ExecStart=/usr/bin/doublezerod -sock-file /run/doublezerod/doublezerod.sock -env testnet -metrics-enable -metrics-addr localhost:2113
```
Metrics will be available at `http://localhost:2113/metrics`.
