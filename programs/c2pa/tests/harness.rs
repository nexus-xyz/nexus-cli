#![cfg(test)]

extern crate alloc;

use alloc::vec::Vec;
use nexus_rt::println;
use nexus_sdk::{precompiles::keccak256, precompiles::ed25519, Local, Prover, Viewable};
use nexus_sdk::stwo::seq::Stwo;

mod test_utils;
use test_utils::{generate_test_keypair, sign_test_data, create_test_image_data};

struct TestProgram {
    prover: Stwo<Local>,
}

impl TestProgram {
    fn new() -> Self {
        let program_path = std::path::Path::new("target/riscv32im-unknown-none-elf/debug/c2pa");
        let prover = Stwo::<Local>::new_from_file(&program_path)
            .expect("Failed to load program");
        Self { prover }
    }

    fn run(&self, public_inputs: &[u8], private_inputs: &[u8]) -> i32 {
        let (view, _proof) = self.prover
            .prove_with_inputs(public_inputs, private_inputs)
            .expect("Failed to run program");
        
        view.exit_code().expect("Failed to get exit code")
    }
}

// Helper to create a test manifest
fn create_test_manifest(
    original_hash: [u8; 32],
    compressed_hash: [u8; 32],
    timestamp: u64,
    private_key: &[u8; 32],
    public_key: &[u8; 32],
    width: u32,
    height: u32,
    quality: u8,
) -> Vec<u8> {
    // Build manifest data
    let mut manifest = Vec::new();
    manifest.extend_from_slice(&original_hash);
    manifest.extend_from_slice(&compressed_hash);
    manifest.extend_from_slice(&timestamp.to_be_bytes());
    manifest.extend_from_slice(public_key);
    
    // Create signature
    let mut payload = Vec::new();
    payload.extend_from_slice(&original_hash);
    payload.extend_from_slice(&compressed_hash);
    payload.extend_from_slice(&timestamp.to_be_bytes());
    payload.extend_from_slice(&[0u8; 8]); // nonce
    payload.extend_from_slice(&width.to_be_bytes());
    payload.extend_from_slice(&height.to_be_bytes());
    payload.extend_from_slice(&[quality]);
    
    let signature = sign_test_data(&keccak256(&payload), private_key);
    
    // Add signature length and data
    manifest.push(signature.len() as u8);
    manifest.extend_from_slice(&signature);
    
    // Add compression params
    manifest.extend_from_slice(&width.to_be_bytes());
    manifest.extend_from_slice(&height.to_be_bytes());
    manifest.push(quality);
    
    manifest
}

// Helper to create program inputs
fn create_program_inputs(
    original_image: &[u8],
    compressed_image: &[u8],
    manifest: &[u8],
) -> (Vec<u8>, Vec<u8>) {
    // Create public inputs
    let mut public_inputs = Vec::new();
    public_inputs.extend_from_slice(&keccak256(compressed_image));
    public_inputs.extend_from_slice(&keccak256(manifest));
    public_inputs.extend_from_slice(&0u64.to_be_bytes()); // nonce
    
    // Create private inputs
    let mut private_inputs = Vec::new();
    
    // Add manifest
    private_inputs.push(manifest.len() as u8);
    private_inputs.extend_from_slice(manifest);
    
    // Add original image
    private_inputs.extend_from_slice(&(original_image.len() as u16).to_be_bytes());
    private_inputs.extend_from_slice(original_image);
    
    // Add compressed image
    private_inputs.extend_from_slice(&(compressed_image.len() as u16).to_be_bytes());
    private_inputs.extend_from_slice(compressed_image);
    
    (public_inputs, private_inputs)
}

#[test]
fn test_valid_compression() {
    let program = TestProgram::new();
    let original = create_test_image_data(100, 100);
    let compressed = create_test_image_data(50, 50);
    let (private_key, public_key) = generate_test_keypair();
    
    let manifest = create_test_manifest(
        keccak256(&original),
        keccak256(&compressed),
        1234567890,
        &private_key,
        &public_key,
        50,
        50,
        80,
    );
    
    let (public_inputs, private_inputs) = create_program_inputs(&original, &compressed, &manifest);
    let exit_code = program.run(&public_inputs, &private_inputs);
    assert_eq!(exit_code, 0, "Valid compression should succeed");
}

#[test]
fn test_invalid_signature() {
    let program = TestProgram::new();
    let original = create_test_image_data(100, 100);
    let compressed = create_test_image_data(50, 50);
    let (_, public_key) = generate_test_keypair();
    let (wrong_private_key, _) = generate_test_keypair();
    
    let manifest = create_test_manifest(
        keccak256(&original),
        keccak256(&compressed),
        1234567890,
        &wrong_private_key,
        &public_key,
        50,
        50,
        80,
    );
    
    let (public_inputs, private_inputs) = create_program_inputs(&original, &compressed, &manifest);
    let exit_code = program.run(&public_inputs, &private_inputs);
    assert_ne!(exit_code, 0, "Invalid signature should fail");
}

#[test]
fn test_invalid_compression_params() {
    let program = TestProgram::new();
    let original = create_test_image_data(100, 100);
    let compressed = create_test_image_data(50, 50);
    let (private_key, public_key) = generate_test_keypair();
    
    let manifest = create_test_manifest(
        keccak256(&original),
        keccak256(&compressed),
        1234567890,
        &private_key,
        &public_key,
        9000,
        9000,
        80,
    );
    
    let (public_inputs, private_inputs) = create_program_inputs(&original, &compressed, &manifest);
    let exit_code = program.run(&public_inputs, &private_inputs);
    assert_ne!(exit_code, 0, "Invalid dimensions should fail");
}

#[test]
fn test_mismatched_image_hash() {
    let program = TestProgram::new();
    let original = create_test_image_data(100, 100);
    let compressed = create_test_image_data(50, 50);
    let (private_key, public_key) = generate_test_keypair();
    
    // Create manifest with wrong hash
    let wrong_hash = [0u8; 32];
    let manifest = create_test_manifest(
        wrong_hash,
        keccak256(&compressed),
        1234567890,
        &private_key,
        &public_key,
        50,
        50,
        80,
    );
    
    let (public_inputs, private_inputs) = create_program_inputs(&original, &compressed, &manifest);
    let exit_code = program.run(&public_inputs, &private_inputs);
    assert_ne!(exit_code, 0, "Mismatched image hash should fail");
}

#[test]
fn test_invalid_quality() {
    let program = TestProgram::new();
    let original = create_test_image_data(100, 100);
    let compressed = create_test_image_data(50, 50);
    let (private_key, public_key) = generate_test_keypair();
    
    let manifest = create_test_manifest(
        keccak256(&original),
        keccak256(&compressed),
        1234567890,
        &private_key,
        &public_key,
        50,
        50,
        101, // Quality > 100
    );
    
    let (public_inputs, private_inputs) = create_program_inputs(&original, &compressed, &manifest);
    let exit_code = program.run(&public_inputs, &private_inputs);
    assert_ne!(exit_code, 0, "Invalid quality should fail");
}

#[test]
fn test_zero_dimensions() {
    let program = TestProgram::new();
    let original = create_test_image_data(100, 100);
    let compressed = create_test_image_data(50, 50);
    let (private_key, public_key) = generate_test_keypair();
    
    let manifest = create_test_manifest(
        keccak256(&original),
        keccak256(&compressed),
        1234567890,
        &private_key,
        &public_key,
        0, // Zero width
        50,
        80,
    );
    
    let (public_inputs, private_inputs) = create_program_inputs(&original, &compressed, &manifest);
    let exit_code = program.run(&public_inputs, &private_inputs);
    assert_ne!(exit_code, 0, "Zero dimensions should fail");
}

#[test]
fn test_timestamp_verification() {
    let program = TestProgram::new();
    let original = create_test_image_data(100, 100);
    let compressed = create_test_image_data(50, 50);
    let (private_key, public_key) = generate_test_keypair();
    
    // Create manifest with future timestamp
    let future_timestamp = u64::MAX;
    let manifest = create_test_manifest(
        keccak256(&original),
        keccak256(&compressed),
        future_timestamp,
        &private_key,
        &public_key,
        50,
        50,
        80,
    );
    
    let (public_inputs, private_inputs) = create_program_inputs(&original, &compressed, &manifest);
    let exit_code = program.run(&public_inputs, &private_inputs);
    // Note: Currently the program doesn't validate timestamps, but we might want to add this
    assert_eq!(exit_code, 0, "Future timestamp currently allowed");
} 