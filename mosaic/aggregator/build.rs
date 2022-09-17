//! Building the gRPC .proto file for handling the MSFLP protocol.
use std::{fs, result::Result};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_pathbuf = fs::canonicalize("../..")?;
    let project_dir = project_pathbuf.to_str().unwrap_or(".");

    tonic_build::configure()
        .build_server(true)
        // .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &[format!("{}/protos/msflp.proto", project_dir)],
            &[format!("{}/protos", project_dir)],
        )?;
    Ok(())
}
