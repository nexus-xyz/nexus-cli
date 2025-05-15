#![no_std]

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

    // Use the zkVM precompile for verification
    ed25519::verify(message, signature, public_key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexus_sdk::precompiles::ed25519;

    // Helper function to generate test data using the host-side precompile
    #[cfg(feature = "host")]
    fn generate_test_data() -> (Vec<u8>, Vec<u8>, Vec<u8>) {
        let message = b"Hello, world!".to_vec();
        let (public_key, private_key) = ed25519::generate_keypair();
        let signature = ed25519::sign(&message, &private_key);
        (message, signature, public_key)
    }

    #[test]
    #[cfg(feature = "host")]
    fn test_verify_valid_signature() {
        let (message, signature, public_key) = generate_test_data();
        assert!(verify_signature(&message, &signature, &public_key));
    }

    #[test]
    #[cfg(feature = "host")]
    fn test_verify_invalid_signature() {
        let (message, _, public_key) = generate_test_data();
        let invalid_signature = [0u8; 64];
        assert!(!verify_signature(&message, &invalid_signature, &public_key));
    }

    #[test]
    #[cfg(feature = "host")]
    fn test_verify_invalid_public_key() {
        let (message, signature, _) = generate_test_data();
        let invalid_public_key = [0u8; 32];
        assert!(!verify_signature(&message, &signature, &invalid_public_key));
    }

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
} 