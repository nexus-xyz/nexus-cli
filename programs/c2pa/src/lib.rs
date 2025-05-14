#![no_std]
#![no_main]

use nexus_rt::println;

#[nexus_rt::main]
fn main() {
    println!("Hello from C2PA program!");
}

// This empty lib.rs file is needed because we specified it in Cargo.toml
// We'll add shared functionality here as we build up the C2PA program 