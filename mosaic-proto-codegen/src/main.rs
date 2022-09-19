extern crate protoc_rust;

use std::{error::Error, fs, path::Path, result::Result};

fn main() -> Result<(), Box<dyn Error>> {
    let project_dir = fs::canonicalize("..")?
        .into_os_string()
        .into_string()
        .unwrap();
    let output_folder = Path::new(&project_dir).join("mosaic/core/src/protos");

    protoc_rust::Codegen::new()
        .out_dir(
            output_folder
                .as_path()
                .to_str()
                .ok_or("Unable to format output path for mosaic-core crate.")?,
        )
        .inputs([
            &format!("{}/protos/dtype.proto", project_dir),
            &format!("{}/protos/tensor_shape.proto", project_dir),
            &format!("{}/protos/tensor.proto", project_dir),
            &format!("{}/protos/msflp.proto", project_dir),
        ])
        .include(project_dir)
        .run()
        .expect("Running protoc failed.");

    Ok(())
}
