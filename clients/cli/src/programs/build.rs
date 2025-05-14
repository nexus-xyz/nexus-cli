use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // Compile the default program
    let status = Command::new("cargo")
        .args(&[
            "build",
            "--release",
            "--target", "riscv32im-unknown-none-elf",
        ])
        .current_dir(&manifest_dir)
        .status()
        .expect("Failed to compile default program");

    assert!(status.success(), "Failed to compile default program");

    // Copy the ELF file to assets
    let elf_source = Path::new(&manifest_dir)
        .join("target")
        .join("riscv32im-unknown-none-elf")
        .join("release")
        .join("c2pa");

    let elf_dest = Path::new(&manifest_dir)
        .parent().unwrap() // src
        .parent().unwrap() // cli
        .join("assets")
        .join("c2pa");

    std::fs::copy(elf_source, elf_dest)
        .expect("Failed to copy ELF file to assets");
} 