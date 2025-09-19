use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // Compile protobuf files
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(&out_dir)
        .compile_protos(
            &[
                "proto/cns.proto",
                "proto/ghostchain.proto",
                "proto/ghostplane.proto",
            ],
            &["proto"],
        )?;

    // Tell cargo to recompile if any proto files change
    println!("cargo:rerun-if-changed=proto/");
    println!("cargo:rerun-if-changed=proto/cns.proto");
    println!("cargo:rerun-if-changed=proto/ghostchain.proto");
    println!("cargo:rerun-if-changed=proto/ghostplane.proto");

    Ok(())
}