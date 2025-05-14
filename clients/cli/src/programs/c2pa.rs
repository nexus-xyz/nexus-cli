#![no_std]
#![no_main]

use core::fmt::Display;
use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

// Error handling
#[derive(Debug)]
enum ProgramError {
    ImageError(&'static str),
    ManifestError(&'static str),
    ValidationError(&'static str),
}

impl Display for ProgramError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProgramError::ImageError(msg) => write!(f, "Image Error: {}", msg),
            ProgramError::ManifestError(msg) => write!(f, "Manifest Error: {}", msg),
            ProgramError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
        }
    }
}

// C2PA manifest structure
#[derive(Serialize, Deserialize, Clone)]
struct C2PAManifest {
    original_hash: String,
    compressed_hash: String,
    timestamp: u64,
    signature: String,
    public_key: String,
    // Additional metadata
    compression_algorithm: String,
    software_agent: String,
    version: String,
}

// Program inputs structure
#[derive(Serialize, Deserialize)]
struct ProgramInput {
    original_image: Vec<u8>,
    compression_params: CompressionParams,
    manifest: C2PAManifest,
    server_nonce: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct CompressionParams {
    target_width: u32,
    target_height: u32,
    quality: u8,
}

#[derive(Serialize)]
struct PublicOutput {
    compressed_image_hash: [u8; 32],
    manifest_hash: [u8; 32],
    server_nonce: u64,
    success: bool,
    error_message: Option<String>,
}

// Image structure for processing
#[derive(Clone)]
struct Image {
    width: u32,
    height: u32,
    data: Vec<u8>, // RGB format
}

risc0_zkvm::guest::entry!(main);

fn main() {
    let result = process_image_and_manifest();
    match result {
        Ok(output) => env::commit(&output),
        Err(e) => {
            let error_output = PublicOutput {
                compressed_image_hash: [0u8; 32],
                manifest_hash: [0u8; 32],
                server_nonce: 0,
                success: false,
                error_message: Some(alloc::format!("{}", e)),
            };
            env::commit(&error_output);
        }
    }
}

fn process_image_and_manifest() -> Result<PublicOutput, ProgramError> {
    // Read and validate input
    let input: ProgramInput = env::read();
    
    // Validate timestamp is within acceptable range
    let current_time = env::get_timestamp();
    if input.manifest.timestamp > current_time + 3600 || input.manifest.timestamp < current_time - 3600 {
        return Err(ProgramError::ValidationError("Timestamp out of valid range"));
    }

    // Parse and validate original image
    let original_image = parse_image(&input.original_image)
        .map_err(|_| ProgramError::ImageError("Failed to parse original image"))?;

    // Verify original image hash
    let original_hash = keccak256(&input.original_image);
    if hex::encode(original_hash) != input.manifest.original_hash {
        return Err(ProgramError::ValidationError("Original image hash mismatch"));
    }

    // Compress image
    let compressed_image = compress_image(&original_image, &input.compression_params)
        .map_err(|_| ProgramError::ImageError("Failed to compress image"))?;
    
    // Verify compressed image hash
    let compressed_bytes = image_to_bytes(&compressed_image);
    let compressed_hash = keccak256(&compressed_bytes);
    if hex::encode(compressed_hash) != input.manifest.compressed_hash {
        return Err(ProgramError::ValidationError("Compressed image hash mismatch"));
    }

    // Verify manifest signature
    verify_manifest_signature(&input.manifest)
        .map_err(|_| ProgramError::ManifestError("Invalid manifest signature"))?;

    // Create manifest hash
    let manifest_bytes = serde_json::to_vec(&input.manifest)
        .map_err(|_| ProgramError::ManifestError("Failed to serialize manifest"))?;
    let manifest_hash = keccak256(&manifest_bytes);

    Ok(PublicOutput {
        compressed_image_hash: compressed_hash,
        manifest_hash,
        server_nonce: input.server_nonce,
        success: true,
        error_message: None,
    })
}

fn keccak256(data: &[u8]) -> [u8; 32] {
    use risc0_zkvm::sha::keccak256;
    keccak256(data)
}

fn parse_image(bytes: &[u8]) -> Result<Image, ProgramError> {
    // Simple image parsing (assuming RGB format)
    // In a real implementation, this would handle various formats
    if bytes.len() < 12 {
        return Err(ProgramError::ImageError("Invalid image data"));
    }

    let width = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let height = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    let data = bytes[8..].to_vec();

    if data.len() != (width * height * 3) as usize {
        return Err(ProgramError::ImageError("Invalid image dimensions"));
    }

    Ok(Image {
        width,
        height,
        data,
    })
}

fn compress_image(image: &Image, params: &CompressionParams) -> Result<Image, ProgramError> {
    if params.target_width == 0 || params.target_height == 0 {
        return Err(ProgramError::ImageError("Invalid target dimensions"));
    }

    let scale_x = (image.width as f32) / (params.target_width as f32);
    let scale_y = (image.height as f32) / (params.target_height as f32);

    let mut compressed = Image {
        width: params.target_width,
        height: params.target_height,
        data: vec![0; (params.target_width * params.target_height * 3) as usize],
    };

    // Bilinear interpolation for downscaling
    for y in 0..params.target_height {
        for x in 0..params.target_width {
            let src_x = (x as f32 * scale_x) as u32;
            let src_y = (y as f32 * scale_y) as u32;
            
            let pixel = get_interpolated_pixel(image, src_x, src_y, scale_x, scale_y)?;
            let dst_idx = ((y * params.target_width + x) * 3) as usize;
            compressed.data[dst_idx..dst_idx + 3].copy_from_slice(&pixel);
        }
    }

    // Apply quality reduction if specified
    if params.quality < 100 {
        apply_quality_reduction(&mut compressed.data, params.quality);
    }

    Ok(compressed)
}

fn get_interpolated_pixel(
    image: &Image,
    x: u32,
    y: u32,
    scale_x: f32,
    scale_y: f32,
) -> Result<[u8; 3], ProgramError> {
    let mut r = 0.0;
    let mut g = 0.0;
    let mut b = 0.0;
    let mut weight_sum = 0.0;

    let x_end = core::cmp::min(x + 2, image.width);
    let y_end = core::cmp::min(y + 2, image.height);

    for sy in y..y_end {
        for sx in x..x_end {
            let weight = pixel_weight(sx as f32 - x as f32, sy as f32 - y as f32);
            let idx = ((sy * image.width + sx) * 3) as usize;
            
            r += weight * image.data[idx] as f32;
            g += weight * image.data[idx + 1] as f32;
            b += weight * image.data[idx + 2] as f32;
            weight_sum += weight;
        }
    }

    if weight_sum == 0.0 {
        return Err(ProgramError::ImageError("Invalid pixel interpolation"));
    }

    Ok([
        (r / weight_sum) as u8,
        (g / weight_sum) as u8,
        (b / weight_sum) as u8,
    ])
}

fn pixel_weight(dx: f32, dy: f32) -> f32 {
    let d = (dx * dx + dy * dy).sqrt();
    if d >= 2.0 {
        0.0
    } else {
        1.0 - (d / 2.0)
    }
}

fn apply_quality_reduction(data: &mut [u8], quality: u8) {
    let factor = quality as f32 / 100.0;
    for pixel in data.iter_mut() {
        let normalized = *pixel as f32 / 255.0;
        let quantized = (normalized * factor * 255.0).round() / factor;
        *pixel = quantized as u8;
    }
}

fn image_to_bytes(image: &Image) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + image.data.len());
    bytes.extend_from_slice(&image.width.to_be_bytes());
    bytes.extend_from_slice(&image.height.to_be_bytes());
    bytes.extend_from_slice(&image.data);
    bytes
}

fn verify_manifest_signature(manifest: &C2PAManifest) -> Result<(), ProgramError> {
    use risc0_zkvm::guest::env::ed25519_verify;

    // Build signing payload
    let payload = format!(
        "{}|{}|{}|{}|{}",
        manifest.original_hash,
        manifest.compressed_hash,
        manifest.timestamp,
        manifest.compression_algorithm,
        manifest.version
    );
    
    let payload_hash = keccak256(payload.as_bytes());
    
    let signature_bytes = hex::decode(&manifest.signature)
        .map_err(|_| ProgramError::ManifestError("Invalid signature format"))?;
    
    let public_key_bytes = hex::decode(&manifest.public_key)
        .map_err(|_| ProgramError::ManifestError("Invalid public key format"))?;

    if !ed25519_verify(&payload_hash, &signature_bytes, &public_key_bytes) {
        return Err(ProgramError::ManifestError("Invalid signature"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::collections::BTreeSet as HashSet;

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
    ) -> C2PAManifest {
        let original_hash = hex::encode(keccak256(original_image));
        let compressed_hash = hex::encode(keccak256(compressed_image));
        
        C2PAManifest {
            original_hash,
            compressed_hash,
            timestamp: 1234567890,
            signature: "test_signature".to_string(),
            public_key: "test_public_key".to_string(),
            compression_algorithm: "bilinear_downscale".to_string(),
            software_agent: "nexus-testnet-iii".to_string(),
            version: "1.0.0".to_string(),
        }
    }

    #[test]
    fn test_image_compression() {
        // Create test image
        let original_image = create_test_image(100, 100);
        
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
        let manifest = create_test_manifest(&original_image, &compressed_bytes);

        // Create program input
        let input = ProgramInput {
            original_image,
            compression_params: params,
            manifest: manifest.clone(),
            server_nonce: 12345,
        };

        // Run the program
        let result = process_image_and_manifest();
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert!(output.error_message.is_none());
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
        let mut original_colors = HashSet::new();
        let mut compressed_colors = HashSet::new();

        for chunk in image.data.chunks(3) {
            original_colors.insert((chunk[0], chunk[1], chunk[2]));
        }

        for chunk in compressed.data.chunks(3) {
            compressed_colors.insert((chunk[0], chunk[1], chunk[2]));
        }

        assert!(compressed_colors.len() < original_colors.len());
    }
} 