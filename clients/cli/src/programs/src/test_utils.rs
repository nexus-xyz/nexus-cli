use crate::c2pa::{C2PAManifest, Image};
use ed25519_dalek::{Keypair, SecretKey};
use sha3::{Digest, Keccak256};

// Standard 32x32 test image with a simple pattern
pub const TEST_IMAGE_WIDTH: u32 = 32;
pub const TEST_IMAGE_HEIGHT: u32 = 32;

pub fn create_standard_test_image() -> Vec<u8> {
    let mut image = Vec::with_capacity(8 + (TEST_IMAGE_WIDTH * TEST_IMAGE_HEIGHT * 3) as usize);
    
    // Add width and height
    image.extend_from_slice(&TEST_IMAGE_WIDTH.to_be_bytes());
    image.extend_from_slice(&TEST_IMAGE_HEIGHT.to_be_bytes());
    
    // Create a simple checkered pattern
    for y in 0..TEST_IMAGE_HEIGHT {
        for x in 0..TEST_IMAGE_WIDTH {
            let is_checker = (x / 8 + y / 8) % 2 == 0;
            let (r, g, b) = if is_checker {
                (255, 255, 255) // White
            } else {
                (0, 0, 0) // Black
            };
            image.extend_from_slice(&[r, g, b]);
        }
    }
    
    image
}

// Standard test keypair for consistent manifests
pub fn get_standard_test_keypair() -> Keypair {
    // Use a fixed seed for reproducible tests
    let seed = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
        17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32
    ];
    let secret = SecretKey::from_bytes(&seed).unwrap();
    Keypair::from_bytes(&secret.to_bytes()).unwrap()
}

pub fn create_standard_test_manifest(
    original_image: &[u8],
    compressed_image: &[u8],
    timestamp: u64,
) -> C2PAManifest {
    let keypair = get_standard_test_keypair();
    
    let original_hash = hex::encode(Keccak256::digest(original_image).as_slice());
    let compressed_hash = hex::encode(Keccak256::digest(compressed_image).as_slice());

    let payload = format!(
        "{}|{}|{}|{}|{}",
        original_hash,
        compressed_hash,
        timestamp,
        "bilinear_downscale",
        "1.0.0"
    );
    
    let signature = keypair.sign(Keccak256::digest(payload.as_bytes()).as_slice());
    
    C2PAManifest {
        original_hash,
        compressed_hash,
        timestamp,
        signature: hex::encode(signature.to_bytes()),
        public_key: hex::encode(keypair.public.to_bytes()),
        compression_algorithm: "bilinear_downscale".to_string(),
        software_agent: "nexus-testnet-iii".to_string(),
        version: "1.0.0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_image_size() {
        let image = create_standard_test_image();
        assert_eq!(image.len(), 8 + (TEST_IMAGE_WIDTH * TEST_IMAGE_HEIGHT * 3) as usize);
    }

    #[test]
    fn test_standard_manifest_reproducibility() {
        let image1 = create_standard_test_image();
        let image2 = create_standard_test_image();
        
        let timestamp = 1234567890;
        
        let manifest1 = create_standard_test_manifest(&image1, &image2, timestamp);
        let manifest2 = create_standard_test_manifest(&image1, &image2, timestamp);
        
        assert_eq!(manifest1.signature, manifest2.signature);
        assert_eq!(manifest1.public_key, manifest2.public_key);
    }
} 