#![no_std]
#![no_main]

use nexus_rt::*;

#[cfg_attr(target_arch = "riscv32", nexus_rt::main)]
#[cfg_attr(not(target_arch = "riscv32"), nexus_rt::custom_output(main))]
pub fn main() -> i32 {
    let signature = input_private_bytes(0..64);
    let public_key = input_private_bytes(64..96);
    let message = input_private_bytes(96..);

    if !ed25519_verify(&message, &signature, &public_key) {
        return 1;
    }

    0
}
