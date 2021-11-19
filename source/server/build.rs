//! Building the gRPC .proto file.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["mosaic.proto"], &["../proto"])?;
    Ok(())
}
