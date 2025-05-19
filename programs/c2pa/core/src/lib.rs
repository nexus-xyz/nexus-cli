#![no_std]

#[cfg(test)]
extern crate alloc;

use cfg_if::cfg_if;

#[cfg(feature = "pure-rust")]
use ed25519_dalek::{Verifier, VerifyingKey, Signature, Signer, SigningKey};

#[cfg(feature = "zkvm")]
use nexus_sdk::precompiles::ed25519;

#[cfg(feature = "pure-rust")]
fn as_fixed_array<const N: usize>(slice: &[u8]) -> Option<&[u8; N]> {
    if slice.len() == N { Some(slice.try_into().unwrap()) } else { None }
}

/// Verify an Ed25519 signature over a message
/// 
/// # Arguments
/// 
/// * `message` - The message to verify
/// * `signature` - The 64-byte Ed25519 signature
/// * `public_key` - The 32-byte Ed25519 public key
/// 
/// # Returns
/// 
/// Returns `true` if the signature is valid, `false` otherwise
pub fn verify_signature(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    if signature.len() != 64 || public_key.len() != 32 {
        return false;
    }

    cfg_if! {
        if #[cfg(feature = "pure-rust")] {
            // Convert raw slices into fixed-size arrays (or return false if conversion fails)
            let (pub_arr, sig_arr) = match (as_fixed_array::<32>(public_key), as_fixed_array::<64>(signature)) {
                (Some(p), Some(s)) => (p, s),
                _ => return false,
            };
            // Convert the raw bytes to the appropriate types
            let verifying_key = match VerifyingKey::from_bytes(pub_arr) {
                Ok(key) => key,
                Err(_) => return false,
            };
            let sig = Signature::from_bytes(sig_arr);
            // Verify the signature using pure Rust implementation
            verifying_key.verify(message, &sig).is_ok()
        } else if #[cfg(feature = "zkvm")] {
            // Use the zkVM precompile for verification
            ed25519::verify(message, signature, public_key)
        } else {
            // No implementation enabled
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;

    #[test]
    fn test_verify_wrong_length_signature() {
        let message = b"Hello, world!";
        let wrong_length_signature = [0u8; 32];
        let public_key = [0u8; 32];
        assert!(!verify_signature(message, &wrong_length_signature, &public_key));
    }

    #[test]
    fn test_verify_wrong_length_public_key() {
        let message = b"Hello, world!";
        let signature = [0u8; 64];
        let wrong_length_public_key = [0u8; 16];
        assert!(!verify_signature(message, &signature, &wrong_length_public_key));
    }

    #[cfg(feature = "pure-rust")]
    #[test]
    fn test_verify_valid_signature() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::OsRng;

        let mut rng = OsRng;
        // (For testing, we use a dummy secret key (all zeros) – in production you'd use a secure random secret.)
        let dummy_secret_bytes = [0u8; 32];
        let signing_key = SigningKey::from_bytes(&dummy_secret_bytes);
        let verifying_key = VerifyingKey::from(&signing_key);

        let message = b"Hello, world!";
        let signature = signing_key.sign(message);

        assert!(verify_signature(
            message,
            signature.to_bytes().as_ref(),
            verifying_key.to_bytes().as_ref()
        ));
    }

    #[cfg(feature = "pure-rust")]
    #[test]
    #[ignore = "Test requires real C2PA manifest data from Starling Lab project"]
    fn test_starling_lab_manifest() {
        // This test is currently skipped until we have real C2PA manifest data
        // from the Starling Lab project. Once we have that data, we'll:
        // 1. Update the test fixtures with real manifest and public key
        // 2. Remove the #[ignore] attribute
        // 3. Verify the signature against real-world data
        let manifest_data = include_bytes!("../../test_fixtures/starling_lab_manifest.bin");
        let public_key = include_bytes!("../../test_fixtures/starling_lab_public_key.bin");
        
        // The manifest contains:
        // 1. Original image hash (32 bytes)
        // 2. Compressed image hash (32 bytes)
        // 3. Timestamp (8 bytes)
        // 4. Public key (32 bytes)
        // 5. Signature length (1 byte)
        // 6. Signature (64 bytes)
        // 7. Compression params (width, height, quality)
        
        // Extract the signature from the manifest
        let signature_start = 32 + 32 + 8 + 32; // Skip original hash, compressed hash, timestamp, public key
        let sig_len = manifest_data[signature_start] as usize;
        let signature = &manifest_data[signature_start + 1..signature_start + 1 + sig_len];
        
        // Extract the message that was signed (everything before the signature)
        let message = &manifest_data[..signature_start];
        
        // Verify the signature
        assert!(verify_signature(message, signature, public_key));
    }
} 