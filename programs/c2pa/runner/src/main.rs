use std::fs;
use std::io::{self, Read};
use std::path::Path;
use sha3::{Digest, Keccak256};

/// Extracts the C2PA manifest from a PNG file (from the caBX chunk)
fn extract_c2pa_manifest_from_png<P: AsRef<Path>>(path: P) -> io::Result<Option<Vec<u8>>> {
    let mut file = fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    // PNG signature is 8 bytes
    if buf.len() < 8 || &buf[..8] != b"\x89PNG\r\n\x1a\n" {
        return Ok(None);
    }
    let mut i = 8;
    while i + 8 <= buf.len() {
        // Each chunk: 4 bytes length, 4 bytes type, data, 4 bytes CRC
        let length = u32::from_be_bytes([buf[i], buf[i+1], buf[i+2], buf[i+3]]) as usize;
        let chunk_type = &buf[i+4..i+8];
        if chunk_type == b"caBX" {
            let start = i + 8;
            let end = start + length;
            if end <= buf.len() {
                return Ok(Some(buf[start..end].to_vec()));
            } else {
                return Ok(None);
            }
        }
        i += 8 + length + 4; // chunk header + data + CRC
    }
    Ok(None)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load the PNG image and extract the C2PA manifest
    let image_path = "cat.png";
    let manifest_data = extract_c2pa_manifest_from_png(image_path)?
        .ok_or("No C2PA manifest found in PNG")?;
    let compressed_image = fs::read(image_path)?;

    // 2. Parse and verify the manifest
    let manifest = c2pa::C2paManifest::parse(&manifest_data)
        .ok_or("Failed to parse manifest")?;
    
    // 3. Verify the compressed image hash
    let mut hasher = Keccak256::new();
    hasher.update(&compressed_image);
    let computed_hash = hasher.finalize().into();
    
    if computed_hash != manifest.compressed_hash {
        println!("❌ Compressed image hash mismatch!");
        println!("Expected: {:?}", manifest.compressed_hash);
        println!("Got:      {:?}", computed_hash);
        return Ok(());
    }

    // 4. Verify the manifest signature
    let valid = manifest.verify(0); // Using 0 as nonce for demo
    println!("Manifest verification: {}", if valid { "✅ SUCCESS" } else { "❌ FAILED" });

    // 5. Print manifest details
    println!("\nManifest Details:");
    println!("Original hash: {:?}", manifest.original_hash);
    println!("Compressed hash: {:?}", manifest.compressed_hash);
    println!("Timestamp: {}", manifest.timestamp);
    println!("Compression params:");
    println!("  Width: {}", manifest.compression_params.target_width);
    println!("  Height: {}", manifest.compression_params.target_height);
    println!("  Quality: {}", manifest.compression_params.quality);

    Ok(())
} 