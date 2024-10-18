use std::io::Result;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../../packages/protobuf");

    std::fs::create_dir_all("src/proto/")?;
    protobuf_codegen::Codegen::new()
        // All inputs and imports from the inputs must reside in `includes` directories.
        .includes(["../../packages/protobuf"])
        // Inputs must reside in some of include paths.
        .input("../../packages/protobuf/infrastructure_map.proto")
        // Specify output directory relative to Cargo output directory.
        .out_dir("src/proto/")
        .run_from_script();
    Ok(())
}
