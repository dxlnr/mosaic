//! Building the gRPC .proto file for handling the MSFLP protocol.
use std::{fs, result::Result};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let project_dir = fs::canonicalize("../..")?
    //     .into_os_string()
    //     .into_string()
    //     .unwrap();
    let project_path = fs::canonicalize("../..")?;
    let project_dir = project_path.to_str().unwrap();
    println!("{:?}", project_dir);

    tonic_build::configure()
        // .build_client(true)
        .build_server(true)
        // .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&[format!("{}/protos/msflp.proto", project_dir)], &[format!("{}/protos", project_dir)])?;
    Ok(())
}