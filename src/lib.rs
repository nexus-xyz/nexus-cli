#![no_std]
#![no_main]

use nexus_sdk::precompiles::{ed25519, input};

#[nexus_rt::main]
pub fn main() -> i32 {
    let signature = input::private_bytes(0..64);
    let public_key = input::private_bytes(64..96);
    let message = input::private_bytes(96..);

    if !ed25519::verify(&message, &signature, &public_key) {
        return 1;
    }

    0
} 