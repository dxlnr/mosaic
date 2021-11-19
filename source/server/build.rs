// fn main() {
//     tonic_build::compile_protos("../proto/mosaic.proto")
//         .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["mosaic.proto"], &["../proto"])?;
    Ok(())
}
