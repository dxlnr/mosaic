/// Error handling while running the process via ['Engine'].
use displaydoc::Display;
use thiserror::Error;

/// Handling state errors when iterating ['Engine'].
#[derive(Debug, Display, Error)]
pub enum StateError {
    /// Some error in the Idle state.
    _Idle,
}
