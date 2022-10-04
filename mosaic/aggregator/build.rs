//! Building the gRPC .proto file for handling the MSFLP protocol.
use std::{fs, result::Result};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_pathbuf = fs::canonicalize("../..")?;
    let project_dir = project_pathbuf.to_str().unwrap_or(".");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(format!("{}/mosaic/core/src/message/grpc/", project_dir))
        .include_file(format!("{}/mosaic/core/src/message/grpc/mod.rs", project_dir))
        // .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &[
                format!("{}/protos/dtype.proto", project_dir),
                format!("{}/protos/tensor_shape.proto", project_dir),
                format!("{}/protos/tensor.proto", project_dir),
                format!("{}/protos/msflp.proto", project_dir)],
            &[format!("{}", project_dir)],
        )?;
    Ok(())
}
