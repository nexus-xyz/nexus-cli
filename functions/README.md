# Nexus CLI Firebase Functions

This directory contains Firebase Cloud Functions for the Nexus CLI project.

## Functions

### `version`
A simple caching proxy that respects the Cache-Control headers from the origin.

- **URL**: `https://us-central1-nexus-cli.cloudfunctions.net/version`
- **Behavior**: Caches responses based on origin's max-age, serves JSON data only
- **Cache Status**: Adds `X-Cache` header (HIT/MISS/STALE) for debugging
- **Fallback**: Serves stale cached data if origin is unavailable

## Development

### Setup
```bash
cd functions
npm install
```

### Local Development
```bash
npm run serve
```

### Build
```bash
npm run build
```

### Deploy
```bash
npm run deploy
```

## Fallback Hierarchy

The CLI uses the following fallback hierarchy for fetching version.json:

1. **Primary**: `https://cli.nexus.xyz/version.json` (Firebase Hosting)
2. **Cache**: `https://us-central1-nexus-cli.cloudfunctions.net/version` (Cloud Function)
3. **Fallback**: `https://raw.githubusercontent.com/nexus-xyz/nexus-cli/refs/heads/main/public/version.json` (GitHub)

This setup helps avoid rate limiting issues while keeping hosting costs low. 