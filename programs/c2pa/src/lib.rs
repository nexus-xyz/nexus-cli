#![no_std]
#![no_main]

use c2pa_core::verify_signature as core_verify;

#[cfg(feature = "zkvm")]
use nexus_sdk::precompiles::ed25519;

pub use c2pa_core::{C2paManifest, CompressionParams};

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
    #[cfg(feature = "pure-rust")]
    {
        core_verify(message, signature, public_key)
    }

    #[cfg(feature = "zkvm")]
    {
        ed25519::verify(message, signature, public_key)
    }
}

#[cfg(feature = "zkvm")]
#[nexus_rt::main]
pub fn main() -> i32 {
    let signature = nexus_sdk::precompiles::input::private_bytes(0..64);
    let public_key = nexus_sdk::precompiles::input::private_bytes(64..96);
    let message = nexus_sdk::precompiles::input::private_bytes(96..);

    if !verify_signature(&message, &signature, &public_key) {
        return 1;
    }

    0
} 