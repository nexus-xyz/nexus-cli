#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::{String, ToString}, vec, vec::Vec, collections::BTreeSet};
use core::{marker::Sized, result::Result, option::Option};
use nexus_sdk::guest::{self, env};
use sha3::{Digest, Keccak256};

// Types
#[derive(Debug, Clone)]
pub struct C2PAManifest {
    pub original_hash: String,
    pub compressed_hash: String,
    pub timestamp: u64,
    pub signature: String,
    pub public_key: String,
    pub compression_algorithm: String,
    pub software_agent: String,
    pub version: String,
}

impl C2PAManifest {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.original_hash.as_bytes());
        bytes.extend_from_slice(b"\0");
        bytes.extend_from_slice(self.compressed_hash.as_bytes());
        bytes.extend_from_slice(b"\0");
        bytes.extend_from_slice(&self.timestamp.to_be_bytes());
        bytes.extend_from_slice(self.signature.as_bytes());
        bytes.extend_from_slice(b"\0");
        bytes.extend_from_slice(self.public_key.as_bytes());
        bytes.extend_from_slice(b"\0");
        bytes.extend_from_slice(self.compression_algorithm.as_bytes());
        bytes.extend_from_slice(b"\0");
        bytes.extend_from_slice(self.software_agent.as_bytes());
        bytes.extend_from_slice(b"\0");
        bytes.extend_from_slice(self.version.as_bytes());
        bytes.extend_from_slice(b"\0");
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut fields = bytes.split(|&b| b == 0);
        
        let original_hash = String::from_utf8(fields.next().ok_or("Missing original_hash")?.to_vec())
            .map_err(|_| "Invalid original_hash")?;
        let compressed_hash = String::from_utf8(fields.next().ok_or("Missing compressed_hash")?.to_vec())
            .map_err(|_| "Invalid compressed_hash")?;
        let timestamp_bytes = fields.next().ok_or("Missing timestamp")?;
        if timestamp_bytes.len() != 8 {
            return Err("Invalid timestamp length");
        }
        let timestamp = u64::from_be_bytes(timestamp_bytes.try_into().unwrap());
        let signature = String::from_utf8(fields.next().ok_or("Missing signature")?.to_vec())
            .map_err(|_| "Invalid signature")?;
        let public_key = String::from_utf8(fields.next().ok_or("Missing public_key")?.to_vec())
            .map_err(|_| "Invalid public_key")?;
        let compression_algorithm = String::from_utf8(fields.next().ok_or("Missing compression_algorithm")?.to_vec())
            .map_err(|_| "Invalid compression_algorithm")?;
        let software_agent = String::from_utf8(fields.next().ok_or("Missing software_agent")?.to_vec())
            .map_err(|_| "Invalid software_agent")?;
        let version = String::from_utf8(fields.next().ok_or("Missing version")?.to_vec())
            .map_err(|_| "Invalid version")?;

        Ok(Self {
            original_hash,
            compressed_hash,
            timestamp,
            signature,
            public_key,
            compression_algorithm,
            software_agent,
            version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CompressionParams {
    pub target_width: u32,
    pub target_height: u32,
    pub quality: u8,
}

impl CompressionParams {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(9);
        bytes.extend_from_slice(&self.target_width.to_be_bytes());
        bytes.extend_from_slice(&self.target_height.to_be_bytes());
        bytes.push(self.quality);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != 9 {
            return Err("Invalid compression params length");
        }

        Ok(Self {
            target_width: u32::from_be_bytes(bytes[0..4].try_into().unwrap()),
            target_height: u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
            quality: bytes[8],
        })
    }
}

pub struct ProgramInput {
    pub original_image: Vec<u8>,
    pub compression_params: CompressionParams,
    pub manifest: C2PAManifest,
    pub server_nonce: u64,
}

impl ProgramInput {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        // Original image length + data
        bytes.extend_from_slice(&(self.original_image.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&self.original_image);
        
        // Compression params
        bytes.extend_from_slice(&self.compression_params.to_bytes());
        
        // Manifest
        let manifest_bytes = self.manifest.to_bytes();
        bytes.extend_from_slice(&(manifest_bytes.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&manifest_bytes);
        
        // Server nonce
        bytes.extend_from_slice(&self.server_nonce.to_be_bytes());
        
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut pos = 0;
        
        // Original image
        if bytes.len() < 4 {
            return Err("Input too short for image length");
        }
        let image_len = u32::from_be_bytes(bytes[0..4].try_into().unwrap()) as usize;
        pos += 4;
        
        if bytes.len() < pos + image_len {
            return Err("Input too short for image data");
        }
        let original_image = bytes[pos..pos + image_len].to_vec();
        pos += image_len;
        
        // Compression params
        if bytes.len() < pos + 9 {
            return Err("Input too short for compression params");
        }
        let compression_params = CompressionParams::from_bytes(&bytes[pos..pos + 9])?;
        pos += 9;
        
        // Manifest
        if bytes.len() < pos + 4 {
            return Err("Input too short for manifest length");
        }
        let manifest_len = u32::from_be_bytes(bytes[pos..pos + 4].try_into().unwrap()) as usize;
        pos += 4;
        
        if bytes.len() < pos + manifest_len {
            return Err("Input too short for manifest data");
        }
        let manifest = C2PAManifest::from_bytes(&bytes[pos..pos + manifest_len])?;
        pos += manifest_len;
        
        // Server nonce
        if bytes.len() < pos + 8 {
            return Err("Input too short for server nonce");
        }
        let server_nonce = u64::from_be_bytes(bytes[pos..pos + 8].try_into().unwrap());
        
        Ok(Self {
            original_image,
            compression_params,
            manifest,
            server_nonce,
        })
    }
}

pub struct ProgramOutput {
    pub success: bool,
    pub error_message: Option<String>,
}

impl ProgramOutput {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.success as u8);
        
        match &self.error_message {
            Some(msg) => {
                bytes.push(1);
                bytes.extend_from_slice(&(msg.len() as u32).to_be_bytes());
                bytes.extend_from_slice(msg.as_bytes());
            }
            None => {
                bytes.push(0);
            }
        }
        
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 2 {
            return Err("Output too short");
        }
        
        let success = bytes[0] != 0;
        let has_error = bytes[1] != 0;
        
        let error_message = if has_error {
            if bytes.len() < 6 {
                return Err("Output too short for error message length");
            }
            let msg_len = u32::from_be_bytes(bytes[2..6].try_into().unwrap()) as usize;
            if bytes.len() < 6 + msg_len {
                return Err("Output too short for error message");
            }
            Some(String::from_utf8(bytes[6..6 + msg_len].to_vec())
                .map_err(|_| "Invalid error message")?)
        } else {
            None
        };
        
        Ok(Self {
            success,
            error_message,
        })
    }
}

// Helper functions
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn parse_image(data: &[u8]) -> Result<Image, &'static str> {
    if data.len() < 8 {
        return Err("Image data too short");
    }

    let width = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    let height = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    
    let expected_size = 8 + (width * height * 3) as usize;
    if data.len() != expected_size {
        return Err("Invalid image data size");
    }

    Ok(Image {
        width,
        height,
        data: data[8..].to_vec(),
    })
}

pub fn compress_image(image: &Image, params: &CompressionParams) -> Result<Image, &'static str> {
    if params.target_width == 0 || params.target_height == 0 {
        return Err("Invalid target dimensions");
    }

    // Simple bilinear interpolation for downscaling
    let mut compressed = Image {
        width: params.target_width,
        height: params.target_height,
        data: vec![0; (params.target_width * params.target_height * 3) as usize],
    };

    let x_ratio = image.width as f32 / params.target_width as f32;
    let y_ratio = image.height as f32 / params.target_height as f32;

    for y in 0..params.target_height {
        for x in 0..params.target_width {
            let px = (x as f32 * x_ratio).floor() as u32;
            let py = (y as f32 * y_ratio).floor() as u32;
            
            let src_idx = ((py * image.width + px) * 3) as usize;
            let dst_idx = ((y * params.target_width + x) * 3) as usize;
            
            for c in 0..3 {
                compressed.data[dst_idx + c] = image.data[src_idx + c];
            }
        }
    }

    // Apply quality reduction if specified
    if params.quality < 100 {
        let factor = params.quality as f32 / 100.0;
        for pixel in compressed.data.chunks_mut(3) {
            for c in pixel.iter_mut() {
                *c = (*c as f32 * factor).round() as u8;
            }
        }
    }

    Ok(compressed)
}

pub fn image_to_bytes(image: &Image) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + image.data.len());
    bytes.extend_from_slice(&image.width.to_be_bytes());
    bytes.extend_from_slice(&image.height.to_be_bytes());
    bytes.extend_from_slice(&image.data);
    bytes
}

pub fn process_image_and_manifest(input: ProgramInput) -> Result<ProgramOutput, &'static str> {
    // Parse and validate the original image
    let image = parse_image(&input.original_image)?;
    
    // Compress the image
    let compressed = compress_image(&image, &input.compression_params)?;
    let compressed_bytes = image_to_bytes(&compressed);
    
    // Verify the manifest hashes
    let original_hash = hex::encode(keccak256(&input.original_image));
    let compressed_hash = hex::encode(keccak256(&compressed_bytes));
    
    if original_hash != input.manifest.original_hash {
        return Err("Original image hash mismatch");
    }
    
    if compressed_hash != input.manifest.compressed_hash {
        return Err("Compressed image hash mismatch");
    }
    
    // Verify the timestamp
    let current_time = guest::get_timestamp();
    if input.manifest.timestamp > current_time {
        return Err("Future timestamp not allowed");
    }
    
    // Verify the server nonce
    if input.server_nonce == 0 {
        return Err("Invalid server nonce");
    }
    
    Ok(ProgramOutput {
        success: true,
        error_message: None,
    })
}

// Entry point
guest::entry!(main);

fn main() {
    // Read the input bytes
    let input_bytes: Vec<u8> = guest::read();
    
    // Parse the input
    let input = match ProgramInput::from_bytes(&input_bytes) {
        Ok(input) => input,
        Err(err) => {
            let output = ProgramOutput {
                success: false,
                error_message: Some(err.to_string()),
            };
            guest::commit(&output.to_bytes());
            return;
        }
    };
    
    // Process the image and manifest
    let output = match process_image_and_manifest(input) {
        Ok(output) => output,
        Err(err) => ProgramOutput {
            success: false,
            error_message: Some(err.to_string()),
        },
    };
    
    // Commit the output
    guest::commit(&output.to_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{create_standard_test_image, create_standard_test_manifest};

    #[test]
    fn test_image_compression() {
        // Use standard test image
        let original_image = create_standard_test_image();
        
        // Create compression parameters
        let params = CompressionParams {
            target_width: 16,  // Half size
            target_height: 16,
            quality: 90,
        };

        // Parse original image
        let image = parse_image(&original_image).unwrap();
        
        // Compress image
        let compressed = compress_image(&image, &params).unwrap();
        let compressed_bytes = image_to_bytes(&compressed);

        // Create manifest with standard timestamp
        let timestamp = 1234567890;
        let manifest = create_standard_test_manifest(&original_image, &compressed_bytes, timestamp);

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
        let original_image = create_standard_test_image();
        
        let params = CompressionParams {
            target_width: 16,
            target_height: 16,
            quality: 90,
        };

        let image = parse_image(&original_image).unwrap();
        let compressed = compress_image(&image, &params).unwrap();
        let compressed_bytes = image_to_bytes(&compressed);

        // Create manifest with wrong timestamp to invalidate signature
        let manifest = create_standard_test_manifest(&original_image, &compressed_bytes, 9999999999);

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
        let original_image = create_standard_test_image();
        let params = CompressionParams {
            target_width: 32,  // Same size
            target_height: 32,
            quality: 50,  // Reduced quality
        };

        let image = parse_image(&original_image).unwrap();
        let compressed = compress_image(&image, &params).unwrap();

        // Verify that the compressed image has reduced quality
        // by checking if the unique color count is less than the original
        let mut original_colors = BTreeSet::new();
        let mut compressed_colors = BTreeSet::new();

        for chunk in image.data.chunks(3) {
            original_colors.insert((chunk[0], chunk[1], chunk[2]));
        }

        for chunk in compressed.data.chunks(3) {
            compressed_colors.insert((chunk[0], chunk[1], chunk[2]));
        }

        assert!(compressed_colors.len() < original_colors.len());
    }

    #[test]
    fn test_invalid_dimensions() {
        let original_image = create_standard_test_image();
        let params = CompressionParams {
            target_width: 0,  // Invalid width
            target_height: 16,
            quality: 90,
        };

        let image = parse_image(&original_image).unwrap();
        let result = compress_image(&image, &params);
        assert!(result.is_err());
    }

    #[test]
    fn test_timestamp_validation() {
        let original_image = create_standard_test_image();
        
        let params = CompressionParams {
            target_width: 16,
            target_height: 16,
            quality: 90,
        };

        let image = parse_image(&original_image).unwrap();
        let compressed = compress_image(&image, &params).unwrap();
        let compressed_bytes = image_to_bytes(&compressed);

        // Create manifest with future timestamp
        let future_timestamp = guest::get_timestamp() + 1000000;
        let manifest = create_standard_test_manifest(&original_image, &compressed_bytes, future_timestamp);

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
    fn test_server_nonce_validation() {
        let original_image = create_standard_test_image();
        
        let params = CompressionParams {
            target_width: 16,
            target_height: 16,
            quality: 90,
        };

        let image = parse_image(&original_image).unwrap();
        let compressed = compress_image(&image, &params).unwrap();
        let compressed_bytes = image_to_bytes(&compressed);

        let manifest = create_standard_test_manifest(&original_image, &compressed_bytes, 1234567890);

        // Test with invalid nonce
        let input = ProgramInput {
            original_image,
            compression_params: params,
            manifest,
            server_nonce: 0, // Invalid nonce
        };

        let result = process_image_and_manifest(input);
        assert!(result.is_err());
    }
} 