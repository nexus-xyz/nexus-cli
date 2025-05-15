#![no_std]
#![cfg(test)]

extern crate alloc;

use alloc::vec::Vec;
use nexus_sdk::precompiles::ed25519;

// Test key generation for manifest verification
pub fn generate_test_keypair() -> ([u8; 32], [u8; 32]) {
    let private_key = [
        157, 97, 177, 157, 239, 253, 90, 96,
        186, 132, 74, 244, 146, 236, 44, 196,
        68, 73, 197, 105, 123, 50, 105, 25,
        112, 59, 172, 3, 28, 174, 127, 96,
    ];
    let public_key = ed25519::derive_public_key(&private_key);
    (private_key, public_key)
}

// Test signature generation
pub fn sign_test_data(data: &[u8], private_key: &[u8; 32]) -> Vec<u8> {
    ed25519::sign(data, private_key)
}

// Helper to create test image data
pub fn create_test_image_data(width: u32, height: u32) -> Vec<u8> {
    let size = (width * height * 3) as usize;
    let mut image = Vec::with_capacity(size);
    for i in 0..size {
        image.push((i % 255) as u8);
    }
    image
} 