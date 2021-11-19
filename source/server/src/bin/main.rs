use server::server::start;
use tracing::warn;
//use tracing_subscriber::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio::select! {
        result = start() => {
            match result {
                Ok(()) => warn!("shutting down: gRPC server terminated."),
                Err(_error) => {
                    warn!("shutting down as error occured.");
                },
            }
        }
    }
    Ok(())
}
