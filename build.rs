use std::fs;
use std::path::Path;

fn main() {
    // Read Cargo.toml and extract the metadata
    let cargo_toml_path = Path::new("Cargo.toml");
    let cargo_toml_content =
        fs::read_to_string(cargo_toml_path).expect("Failed to read Cargo.toml");

    // Parse the TOML
    let cargo_toml: toml::Value =
        toml::from_str(&cargo_toml_content).expect("Failed to parse Cargo.toml");

    // Extract the metadata
    if let Some(metadata) = cargo_toml
        .get("package")
        .and_then(|p| p.get("metadata"))
        .and_then(|m| m.get("settings"))
        .and_then(|s| s.get("defaults"))
    {
        // Convert metadata to JSON for easier parsing in the main code
        let metadata_json = serde_json::to_string(metadata).expect("Failed to serialize metadata");

        // Set environment variable for use in main code
        println!("cargo:rustc-env=PACKAGE_METADATA_JSON={}", metadata_json);
    }

    // Tell cargo to re-run if Cargo.toml changes
    println!("cargo:rerun-if-changed=Cargo.toml");
}
