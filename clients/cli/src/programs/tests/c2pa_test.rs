#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Keypair, Signer};
    use rand::rngs::OsRng;

    fn create_test_image(width: u32, height: u32) -> Vec<u8> {
        let mut image = Vec::with_capacity(8 + (width * height * 3) as usize);
        image.extend_from_slice(&width.to_be_bytes());
        image.extend_from_slice(&height.to_be_bytes());
        
        // Create a simple gradient pattern
        for y in 0..height {
            for x in 0..width {
                let r = ((x as f32 / width as f32) * 255.0) as u8;
                let g = ((y as f32 / height as f32) * 255.0) as u8;
                let b = ((x as f32 + y as f32) / ((width + height) as f32) * 255.0) as u8;
                image.extend_from_slice(&[r, g, b]);
            }
        }
        
        image
    }

    fn create_test_manifest(
        original_image: &[u8],
        compressed_image: &[u8],
        keypair: &Keypair,
    ) -> C2PAManifest {
        let original_hash = hex::encode(keccak256(original_image));
        let compressed_hash = hex::encode(keccak256(compressed_image));
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let payload = format!(
            "{}|{}|{}|{}|{}",
            original_hash,
            compressed_hash,
            timestamp,
            "bilinear_downscale",
            "1.0.0"
        );
        
        let signature = keypair.sign(keccak256(payload.as_bytes()).as_ref());
        
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

    #[test]
    fn test_image_compression() {
        // Create test image
        let original_image = create_test_image(100, 100);
        
        // Generate keypair for signing
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);

        // Create compression parameters
        let params = CompressionParams {
            target_width: 50,
            target_height: 50,
            quality: 90,
        };

        // Parse original image
        let image = parse_image(&original_image).unwrap();
        
        // Compress image
        let compressed = compress_image(&image, &params).unwrap();
        let compressed_bytes = image_to_bytes(&compressed);

        // Create manifest
        let manifest = create_test_manifest(&original_image, &compressed_bytes, &keypair);

        // Create program input
        let input = ProgramInput {
            original_image,
            compression_params: params,
            manifest,
            server_nonce: 12345,
        };

        // Run the program
        let result = process_image_and_manifest(input);
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.error_message.is_none());
    }

    #[test]
    fn test_invalid_signature() {
        let original_image = create_test_image(100, 100);
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        let wrong_keypair = Keypair::generate(&mut csprng);

        let params = CompressionParams {
            target_width: 50,
            target_height: 50,
            quality: 90,
        };

        let image = parse_image(&original_image).unwrap();
        let compressed = compress_image(&image, &params).unwrap();
        let compressed_bytes = image_to_bytes(&compressed);

        // Create manifest with wrong keypair
        let manifest = create_test_manifest(&original_image, &compressed_bytes, &wrong_keypair);

        let input = ProgramInput {
            original_image,
            compression_params: params,
            manifest,
            server_nonce: 12345,
        };

        let result = process_image_and_manifest(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_quality_reduction() {
        let original_image = create_test_image(100, 100);
        let params = CompressionParams {
            target_width: 100,
            target_height: 100,
            quality: 50,
        };

        let image = parse_image(&original_image).unwrap();
        let compressed = compress_image(&image, &params).unwrap();

        // Verify that the compressed image has reduced quality
        // by checking if the unique color count is less than the original
        let mut original_colors = std::collections::HashSet::new();
        let mut compressed_colors = std::collections::HashSet::new();

        for chunk in image.data.chunks(3) {
            original_colors.insert((chunk[0], chunk[1], chunk[2]));
        }

        for chunk in compressed.data.chunks(3) {
            compressed_colors.insert((chunk[0], chunk[1], chunk[2]));
        }

        assert!(compressed_colors.len() < original_colors.len());
    }
} 