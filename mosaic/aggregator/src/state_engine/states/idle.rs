use async_trait::async_trait;
use tracing::warn;

#[derive(Debug)]
/// [`Idle`] state.
pub struct Idle;