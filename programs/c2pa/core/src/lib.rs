#![no_std]

use cfg_if::cfg_if;

#[cfg(feature = "pure-rust")]
use ed25519_dalek::{Verifier, VerifyingKey, Signature};

#[cfg(feature = "zkvm")]
use nexus_sdk::precompiles::ed25519;

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
            // Convert the raw bytes to the appropriate types
            let verifying_key = match VerifyingKey::from_bytes(public_key) {
                Ok(key) => key,
                Err(_) => return false,
            };

            let sig = match Signature::from_bytes(signature) {
                Ok(sig) => sig,
                Err(_) => return false,
            };

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
        use ed25519_dalek::{SigningKey, VerifyingKey};
        use rand::rngs::OsRng;

        let mut rng = OsRng;
        let signing_key = SigningKey::generate(&mut rng);
        let verifying_key = VerifyingKey::from(&signing_key);
        
        let message = b"Hello, world!";
        let signature = signing_key.sign(message);
        
        assert!(verify_signature(
            message,
            signature.to_bytes().as_ref(),
            verifying_key.to_bytes().as_ref()
        ));
    }
} 