#![no_std]
#![cfg(test)]

extern crate alloc;

use alloc::vec::Vec;
use c2pa::{C2paManifest, CompressionParams};
mod test_utils;
use test_utils::{generate_test_keypair, sign_test_data};

fn create_test_manifest() -> (Vec<u8>, [u8; 32]) {
    let mut data = Vec::new();
    let (private_key, public_key) = generate_test_keypair();
    
    // Original hash (32 bytes)
    data.extend_from_slice(&[1u8; 32]);
    
    // Compressed hash (32 bytes)
    data.extend_from_slice(&[2u8; 32]);
    
    // Timestamp (8 bytes)
    data.extend_from_slice(&1234567890u64.to_be_bytes());
    
    // Public key (32 bytes)
    data.extend_from_slice(&public_key);
    
    // Create signature
    let mut payload = Vec::new();
    payload.extend_from_slice(&[1u8; 32]); // original_hash
    payload.extend_from_slice(&[2u8; 32]); // compressed_hash
    payload.extend_from_slice(&1234567890u64.to_be_bytes()); // timestamp
    payload.extend_from_slice(&12345u64.to_be_bytes()); // test nonce
    payload.extend_from_slice(&800u32.to_be_bytes()); // width
    payload.extend_from_slice(&600u32.to_be_bytes()); // height
    payload.extend_from_slice(&[80]); // quality
    
    let signature = sign_test_data(&payload, &private_key);
    
    // Signature length (1 byte)
    data.push(signature.len() as u8);
    
    // Signature
    data.extend_from_slice(&signature);
    
    // Compression params
    data.extend_from_slice(&800u32.to_be_bytes()); // width
    data.extend_from_slice(&600u32.to_be_bytes()); // height
    data.push(80); // quality
    
    (data, public_key)
}

#[test]
fn test_manifest_parse() {
    let (data, public_key) = create_test_manifest();
    let manifest = C2paManifest::parse(&data).expect("Failed to parse manifest");
    
    assert_eq!(manifest.original_hash, [1u8; 32]);
    assert_eq!(manifest.compressed_hash, [2u8; 32]);
    assert_eq!(manifest.timestamp, 1234567890);
    assert_eq!(manifest.public_key, public_key);
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
    let (data, _) = create_test_manifest();
    let manifest = C2paManifest::parse(&data)
        .expect("Failed to parse manifest");
    
    // Verify with the test nonce we used to create the signature
    assert!(manifest.verify(12345));
    
    // Verify fails with different nonce
    assert!(!manifest.verify(54321));
} 