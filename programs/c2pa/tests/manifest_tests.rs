#![cfg(test)]

extern crate alloc;

use alloc::vec::Vec;
use c2pa::{C2paManifest, CompressionParams};

fn create_test_manifest() -> Vec<u8> {
    let mut data = Vec::new();
    
    // Original hash (32 bytes)
    data.extend_from_slice(&[1u8; 32]);
    
    // Compressed hash (32 bytes)
    data.extend_from_slice(&[2u8; 32]);
    
    // Timestamp (8 bytes)
    data.extend_from_slice(&1234567890u64.to_be_bytes());
    
    // Public key (32 bytes)
    data.extend_from_slice(&[3u8; 32]);
    
    // Signature length (1 byte)
    data.push(64);
    
    // Signature (64 bytes)
    data.extend_from_slice(&[4u8; 64]);
    
    // Compression params
    data.extend_from_slice(&800u32.to_be_bytes()); // width
    data.extend_from_slice(&600u32.to_be_bytes()); // height
    data.push(80); // quality
    
    data
}

#[test]
fn test_manifest_parse() {
    let data = create_test_manifest();
    let manifest = C2paManifest::parse(&data).expect("Failed to parse manifest");
    
    assert_eq!(manifest.original_hash, [1u8; 32]);
    assert_eq!(manifest.compressed_hash, [2u8; 32]);
    assert_eq!(manifest.timestamp, 1234567890);
    assert_eq!(manifest.public_key, [3u8; 32]);
    assert_eq!(manifest.signature, vec![4u8; 64]);
    assert_eq!(manifest.compression_params.target_width, 800);
    assert_eq!(manifest.compression_params.target_height, 600);
    assert_eq!(manifest.compression_params.quality, 80);
}

#[test]
fn test_manifest_parse_invalid_length() {
    let data = vec![0u8; 50]; // Too short
    assert!(C2paManifest::parse(&data).is_none());
}

#[test]
fn test_manifest_verify() {
    // TODO: Add test with real Ed25519 signature
    // For now just testing basic payload construction
    let manifest = C2paManifest::parse(&create_test_manifest())
        .expect("Failed to parse manifest");
    
    // This will fail because we're using dummy signature data
    assert!(!manifest.verify(12345));
}

#[test]
fn test_manifest_verify_with_valid_signature() {
    // TODO: Generate valid Ed25519 keypair and signature
    // Test with real cryptographic verification
} 