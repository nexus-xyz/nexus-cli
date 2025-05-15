# C2PA Image Compression Verification Program

This program is part of the Nexus Testnet III default program suite, demonstrating the capabilities of the Nexus zkVM for verifying image compression with C2PA manifests.

## Overview

The program verifies that an image has been correctly compressed while maintaining a valid C2PA (Coalition for Content Provenance and Authenticity) manifest chain. It's designed to work with public domain images from trusted sources like NASA archives and Wikimedia Commons.

### Key Features

- Verifies image compression parameters (dimensions, quality)
- Validates C2PA manifest integrity
- Uses Keccak256 for cryptographic hashing
- Supports Ed25519 signatures for manifest verification
- Prevents replay attacks using server nonces

## Technical Details

### Input Format

The program accepts both public and private inputs:

#### Public Inputs
```
[compressed_image_hash(32)][manifest_hash(32)][nonce(8)]
```

#### Private Inputs
```
[manifest_len(1)][manifest_data(N)][orig_img_len(2)][orig_img_data(M)][comp_img_len(2)][comp_img_data(K)]
```

### C2PA Manifest Format
```
[orig_hash(32)][comp_hash(32)][timestamp(8)][pubkey(32)][sig_len(1)][signature(N)][width(4)][height(4)][quality(1)]
```

### Compression Parameters

- Maximum dimensions: 8192x8192 pixels
- Quality range: 0-100
- Output format: Custom format with metadata header

## Usage

1. **Prepare Inputs**
   ```rust
   // Example of preparing inputs
   let manifest = C2paManifest {
       original_hash: [/* 32 bytes */],
       compressed_hash: [/* 32 bytes */],
       timestamp: /* unix timestamp */,
       // ... other fields
   };
   ```

2. **Run Verification**
   ```rust
   // The program will:
   // 1. Verify manifest signature
   // 2. Check compression parameters
   // 3. Validate image hashes
   // 4. Ensure proper compression
   ```

## Testing

Run the test suite:
```bash
cargo test
```

The test suite includes:
- Manifest parsing and validation
- Compression parameter verification
- Edge cases and error handling
- Cryptographic signature verification

## Security Considerations

- Uses Keccak256 instead of SHA-256 for performance
- Implements nonce-based replay protection
- Validates all compression parameters
- Verifies cryptographic signatures

## Integration

This program is designed to work with the Nexus zkVM ecosystem. It can be integrated into larger systems that need to verify image transformations while maintaining provenance information.

## Contributing

When contributing to this program:
1. Ensure all tests pass
2. Add tests for new functionality
3. Follow the existing code style
4. Update documentation as needed

## License

This program is part of the Nexus zkVM project and is licensed under the same terms as the main project. 