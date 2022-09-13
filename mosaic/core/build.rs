use std::io::Result;

fn main() -> Result<()> {
    prost_build::compile_protos(&["src/proto/dtype.proto", "src/proto/tensor_shape.proto", "src/proto/tensor.proto"], &["./src"])?;
    Ok(())
}