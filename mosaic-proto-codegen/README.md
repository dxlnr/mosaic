<h1 align="center">
  <b>Codegen for GRPC messaging protocol</b><br>
</h1>

This crates provides automated code generation for the data structures used in the grpc protocol. This is necessary when de-/encoding
the data types that are sent around between client and server.

This crate relies on [protoc_rust](https://docs.rs/protoc-rust/latest/protoc_rust/index.html) which is an API to generate .rs files using protoc to parse files. 

## Running mosaic-proto-codegen
```shell
cargo run
```