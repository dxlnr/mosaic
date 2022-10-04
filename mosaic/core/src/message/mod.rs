//! [`Message`] implementation for the Modalic protocol.
//!
//! There will be different types of messages in the future.
pub mod grpc;

#[derive(Debug, Eq, PartialEq, Clone)]
/// A header common to all messages.
pub struct Message {}
