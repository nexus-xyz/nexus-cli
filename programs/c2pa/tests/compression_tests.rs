#![cfg(test)]

use c2pa::CompressionParams;

fn create_test_image(width: u32, height: u32) -> Vec<u8> {
    let size = (width * height * 3) as usize;
    let mut image = Vec::with_capacity(size);
    for i in 0..size {
        image.push((i % 255) as u8);
    }
    image
}

#[test]
fn test_compression_params_validation() {
    let valid_params = CompressionParams {
        target_width: 800,
        target_height: 600,
        quality: 80,
    };
    assert!(valid_params.validate());

    let invalid_width = CompressionParams {
        target_width: 0,
        target_height: 600,
        quality: 80,
    };
    assert!(!invalid_width.validate());

    let invalid_height = CompressionParams {
        target_width: 800,
        target_height: 0,
        quality: 80,
    };
    assert!(!invalid_height.validate());

    let invalid_quality = CompressionParams {
        target_width: 800,
        target_height: 600,
        quality: 101,
    };
    assert!(!invalid_quality.validate());

    let max_dimensions = CompressionParams {
        target_width: 8192,
        target_height: 8192,
        quality: 100,
    };
    assert!(max_dimensions.validate());

    let too_large = CompressionParams {
        target_width: 8193,
        target_height: 8192,
        quality: 100,
    };
    assert!(!too_large.validate());
}

#[test]
fn test_compression() {
    let original = create_test_image(1920, 1080);
    let params = CompressionParams {
        target_width: 800,
        target_height: 600,
        quality: 80,
    };

    let compressed = c2pa::compress_image(&original, &params)
        .expect("Compression failed");

    assert!(compressed.len() < original.len());
    assert!(c2pa::verify_compression(&original, &compressed, &params));
}

#[test]
fn test_compression_verify_dimensions() {
    let original = create_test_image(1920, 1080);
    let params = CompressionParams {
        target_width: 800,
        target_height: 600,
        quality: 80,
    };

    let compressed = c2pa::compress_image(&original, &params)
        .expect("Compression failed");

    // Try to verify with wrong dimensions
    let wrong_params = CompressionParams {
        target_width: 640,
        target_height: 480,
        quality: 80,
    };
    assert!(!c2pa::verify_compression(&original, &compressed, &wrong_params));
}

#[test]
fn test_compression_verify_quality() {
    let original = create_test_image(1920, 1080);
    let params = CompressionParams {
        target_width: 800,
        target_height: 600,
        quality: 80,
    };

    let compressed = c2pa::compress_image(&original, &params)
        .expect("Compression failed");

    // Try to verify with wrong quality
    let wrong_params = CompressionParams {
        target_width: 800,
        target_height: 600,
        quality: 90,
    };
    assert!(!c2pa::verify_compression(&original, &compressed, &wrong_params));
}

#[test]
fn test_compression_invalid_params() {
    let original = create_test_image(1920, 1080);
    let invalid_params = CompressionParams {
        target_width: 0,
        target_height: 600,
        quality: 80,
    };

    assert!(c2pa::compress_image(&original, &invalid_params).is_none());
} 