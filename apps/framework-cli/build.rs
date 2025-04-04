use std::io::Result;
use std::process::Command;

fn package_templates() -> Result<()> {
    let root_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let package_templates_dir = root_dir.join("scripts/package-templates.js");

    Command::new(&package_templates_dir)
        .current_dir(root_dir)
        .status()?;
    Ok(())
}

fn main() -> Result<()> {
    // Package templates first
    println!("cargo:rerun-if-changed=../../templates");
    package_templates()?;

    println!("cargo:rerun-if-changed=../../packages/protobuf");

    // Pass PostHog API key from environment variable at build time
    if let Ok(posthog_api_key) = std::env::var("POSTHOG_API_KEY") {
        println!("cargo:rustc-env=POSTHOG_API_KEY={}", posthog_api_key);
    }
    println!("cargo:rerun-if-env-changed=POSTHOG_API_KEY");

    std::fs::create_dir_all("src/proto/")?;
    protobuf_codegen::Codegen::new()
        .pure()
        // All inputs and imports from the inputs must reside in `includes` directories.
        .includes(["../../packages/protobuf"])
        // Inputs must reside in some of include paths.
        .input("../../packages/protobuf/infrastructure_map.proto")
        // Specify output directory relative to Cargo output directory.
        .out_dir("src/proto/")
        .run_from_script();
    Ok(())
}
