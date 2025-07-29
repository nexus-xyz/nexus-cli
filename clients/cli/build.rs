use prost_build::Config;
use std::error::Error;
use std::fs;
use std::process::Command;
use std::{env, path::Path};

/// Compiles the protobuf files into Rust code using prost-build.
fn main() -> Result<(), Box<dyn Error>> {
    // Set build timestamp in milliseconds since epoch
    let build_timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);

    // Skip proto compilation unless build_proto feature is enabled.
    if !cfg!(feature = "build_proto") {
        println!(
            "cargo:warning=Skipping proto compilation. Enable with `cargo clean && cargo build --features build_proto`"
        );
        return Ok(());
    }

    // Tell cargo to recompile if any of these files change.
    println!("cargo:rerun-if-changed=../../proto/orchestrator.proto");
    println!("cargo:rerun-if-changed=build.rs");

    // The output directory for generated Rust files.
    let out_dir = "src/proto";

    // Re-run if the generated file does not yet exist, e.g., was deleted.
    let generated_file_path = format!("{}/nexus.orchestrator.rs", out_dir);
    if !Path::new(&generated_file_path).exists() {
        println!("cargo:warning=Generated file not found, re-running build script.");
        // Checking a file that does not exist (i.e., NULL) always triggers a rerun
        println!("cargo:rerun-if-changed=NULL");
    }

    let mut config = Config::new();
    config.protoc_arg("-I../../proto");
    config.out_dir(out_dir);
    // Add the experimental flag for proto3 optional fields
    config.protoc_arg("--experimental_allow_proto3_optional");

    // Print current directory
    println!("Current dir: {:?}", env::current_dir()?);

    // Check if proto file exists
    let proto_path = Path::new("../../proto/orchestrator.proto");
    if !proto_path.exists() {
        println!("Proto file not found at: {:?}", proto_path);
        return Err("Proto file not found".into());
    }

    // Check if protoc is installed and accessible
    let output = Command::new("which")
        .arg("protoc")
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        println!("protoc is installed and accessible.");
    } else {
        println!("Error: protoc is not installed or not in PATH.");
        return Err("protoc not found".into());
    }

    // Check if the output directory exists and is writable
    if fs::metadata(out_dir).is_ok() {
        println!("Output directory {} exists.", out_dir);
    } else {
        println!("Error: Output directory {} does not exist.", out_dir);
        // Attempt to create the directory if it doesn't exist
        fs::create_dir_all(out_dir)?;
        println!("Created output directory {}.", out_dir);
    }

    // Attempt to compile the .proto file
    match config.compile_protos(&["../../proto/orchestrator.proto"], &["proto"]) {
        Ok(_) => {
            println!("Successfully compiled protobuf files.");
        }
        Err(e) => {
            println!("Error compiling protobuf files: {}", e);
            // Log more details about the error
            match e.kind() {
                std::io::ErrorKind::NotFound => {
                    println!("Error: Could not find a necessary file or directory.");
                }
                _ => println!("Error: {}", e),
            }
            return Err(Box::new(e));
        }
    }

    // Print where the generated file is saved
    println!("Generated file saved to: {}", generated_file_path);

    // Check if the generated file exists
    if fs::metadata(&generated_file_path).is_err() {
        return Err("Generated file does not exist".into());
    }

    Ok(())
}
