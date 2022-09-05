//! Mosaic Server.
//!
//! This binary serves as entry point for the server implementation and executes it.
//! Every single instance is designed for performing an individual training process
//! and terminates when finished.

use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
