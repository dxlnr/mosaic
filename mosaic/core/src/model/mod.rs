//! ML Model representation.
//! 
pub mod tensor;

pub struct Model {
    /// Actual ['Model'] content.
    pub tensor: u32,
    /// Model version which returns the round_id in which the local model was trained 
    /// or aggregated by the server.
    pub model_version: u32,
}