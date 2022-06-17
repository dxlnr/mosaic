//! Handling the connection from client to server to state engine.
//!
//! Serves as an intermediate proxy handling the incoming messages from client to the engine.
//! The engine then processes the messages according to the aggregation strategy.
pub mod message;
pub mod server;
