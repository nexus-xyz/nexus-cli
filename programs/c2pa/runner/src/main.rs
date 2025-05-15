fn main() {
    let message = b"Hello, world!";
    let signature = [0u8; 64];
    let public_key = [0u8; 32];
    let valid = c2pa_core::verify_signature(message, &signature, &public_key);
    println!("Signature valid: {}", valid);
} 