//! ML Model representation.
//!
pub mod serde;
pub mod tensor;

use crate::model::tensor::Tensor;
use std::slice::Iter;

/// [`Model`] represents a Machine Learning model, adapted to FL.
///
#[derive(Debug, Clone, Default)]
pub struct Model {
    /// Actual ['Model'] content.
    pub tensors: Vec<Tensor>,
    /// Model version which returns the round_id in which the local model was trained
    /// or aggregated by the server.
    pub model_version: u32,
}

impl Model {
    pub fn new(tensors: Vec<Tensor>, model_version: u32) -> Self {
        Self {
            tensors,
            model_version,
        }
    }
    /// Returns the number of single tensors within a model.
    pub fn len(&self) -> usize {
        self.tensors.len()
    }
    /// Creates an iterator that yields references to the weights/parameters of this model.
    pub fn iter(&self) -> Iter<Tensor> {
        self.tensors.iter()
    }
}
