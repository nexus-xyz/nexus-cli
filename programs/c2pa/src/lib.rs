#![no_std]
#![no_main]

extern crate alloc;

mod compression;

use alloc::{string::String, vec::Vec};
use nexus_rt::println;
use nexus_sdk::{precompile, ed25519};
use compression::{CompressionParams, verify_compression};

// C2PA manifest structure
#[derive(Default)]
struct C2paManifest {
    original_hash: [u8; 32],
    compressed_hash: [u8; 32],
    timestamp: u64,
    signature: Vec<u8>,
    public_key: [u8; 32],
    compression_params: CompressionParams,
}

impl C2paManifest {
    fn verify(&self, nonce: u64) -> bool {
        // Build signing payload
        let mut payload = Vec::new();
        payload.extend_from_slice(&self.original_hash);
        payload.extend_from_slice(&self.compressed_hash);
        payload.extend_from_slice(&self.timestamp.to_be_bytes());
        payload.extend_from_slice(&nonce.to_be_bytes());
        
        // Add compression parameters to payload
        payload.extend_from_slice(&self.compression_params.target_width.to_be_bytes());
        payload.extend_from_slice(&self.compression_params.target_height.to_be_bytes());
        payload.extend_from_slice(&[self.compression_params.quality]);
        
        // Hash the payload
        let payload_hash = precompile::keccak256(&payload);
        
        // Verify signature
        ed25519::verify(&payload_hash, &self.signature, &self.public_key)
    }

    fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 113 { // 32 + 32 + 8 + 32 + 1 + 8 bytes minimum
            return None;
        }

        let mut offset = 0;
        let mut manifest = Self::default();

        // Parse original hash
        manifest.original_hash.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        // Parse compressed hash
        manifest.compressed_hash.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        // Parse timestamp
        let mut timestamp_bytes = [0u8; 8];
        timestamp_bytes.copy_from_slice(&data[offset..offset + 8]);
        manifest.timestamp = u64::from_be_bytes(timestamp_bytes);
        offset += 8;

        // Parse public key
        manifest.public_key.copy_from_slice(&data[offset..offset + 32]);
        offset += 32;

        // Parse signature length
        let sig_len = data[offset] as usize;
        offset += 1;

        if data.len() < offset + sig_len + 12 {
            return None;
        }

        // Parse signature
        manifest.signature = data[offset..offset + sig_len].to_vec();
        offset += sig_len;

        // Parse compression params
        let mut width_bytes = [0u8; 4];
        width_bytes.copy_from_slice(&data[offset..offset + 4]);
        manifest.compression_params.target_width = u32::from_be_bytes(width_bytes);
        offset += 4;

        let mut height_bytes = [0u8; 4];
        height_bytes.copy_from_slice(&data[offset..offset + 4]);
        manifest.compression_params.target_height = u32::from_be_bytes(height_bytes);
        offset += 4;

        manifest.compression_params.quality = data[offset];

        Some(manifest)
    }
}

#[nexus_rt::main]
fn main() {
    println!("C2PA Manifest Validation Program");

    // Get public inputs
    let public_inputs = nexus_rt::public_inputs();
    if public_inputs.len() < 74 { // 32 + 32 + 8 + 2 bytes minimum
        println!("Invalid public inputs length");
        return;
    }

    // Parse inputs
    let mut offset = 0;
    
    // Get compressed image hash from public inputs
    let mut compressed_hash = [0u8; 32];
    compressed_hash.copy_from_slice(&public_inputs[offset..offset + 32]);
    offset += 32;

    // Get manifest hash from public inputs
    let mut manifest_hash = [0u8; 32];
    manifest_hash.copy_from_slice(&public_inputs[offset..offset + 32]);
    offset += 32;

    // Get nonce from public inputs
    let mut nonce_bytes = [0u8; 8];
    nonce_bytes.copy_from_slice(&public_inputs[offset..offset + 8]);
    let nonce = u64::from_be_bytes(nonce_bytes);
    
    // Get private inputs (original image and manifest)
    let private_inputs = nexus_rt::private_inputs();
    
    // Parse manifest from private inputs
    let manifest_data_len = private_inputs[0] as usize;
    let manifest_data = &private_inputs[1..1 + manifest_data_len];
    
    let manifest = match C2paManifest::parse(manifest_data) {
        Some(m) => m,
        None => {
            println!("Failed to parse manifest");
            return;
        }
    };
    
    // Verify manifest
    if !manifest.verify(nonce) {
        println!("Manifest verification failed");
        return;
    }
    
    // Get original and compressed images from private inputs
    let original_image_offset = 1 + manifest_data_len;
    let original_image_len = ((private_inputs[original_image_offset] as u16) << 8 | 
                             private_inputs[original_image_offset + 1] as u16) as usize;
    let original_image = &private_inputs[original_image_offset + 2..
                                       original_image_offset + 2 + original_image_len];
    
    let compressed_image_offset = original_image_offset + 2 + original_image_len;
    let compressed_image_len = ((private_inputs[compressed_image_offset] as u16) << 8 | 
                               private_inputs[compressed_image_offset + 1] as u16) as usize;
    let compressed_image = &private_inputs[compressed_image_offset + 2..
                                         compressed_image_offset + 2 + compressed_image_len];
    
    // Verify compression
    if !verify_compression(original_image, compressed_image, &manifest.compression_params) {
        println!("Compression verification failed");
        return;
    }
    
    // Verify hashes match
    let calc_original_hash = precompile::keccak256(original_image);
    let calc_compressed_hash = precompile::keccak256(compressed_image);
    
    if calc_original_hash != manifest.original_hash || 
       calc_compressed_hash != manifest.compressed_hash {
        println!("Image hash mismatch");
        return;
    }
    
    println!("Verification successful!");
}

// This empty lib.rs file is needed because we specified it in Cargo.toml
// We'll add shared functionality here as we build up the C2PA program 