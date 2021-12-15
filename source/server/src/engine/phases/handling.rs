use async_trait::async_trait;
use std::convert::Infallible;

use crate::engine::tunnel::EngineRequest;
/// A trait that must be implemented by a state to handle a request.
#[async_trait]
pub trait Handler {
    /// Handling a request.
    async fn handle_request(&mut self, req: EngineRequest) -> Result<(), Infallible>;
}
