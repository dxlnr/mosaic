//! ML Model representation.
//!
pub mod serde;
pub mod tensor;

use crate::model::tensor::Tensor;
use std::slice::Iter;

/// [`Model`] represents a Machine Learning model, adapted to FL.
///
#[derive(Default)]
pub struct Model<T> {
    /// Actual ['Model'] content.
    pub tensors: Vec<Tensor<T>>,
    /// Model version which returns the round_id in which the local model was trained
    /// or aggregated by the server.
    pub model_version: u32,
}

impl<T> Model<T> {
    pub fn new(tensors: Vec<Tensor<T>>, model_version: u32) -> Self {
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
    pub fn iter(&self) -> Iter<Tensor<T>> {
        self.tensors.iter()
    }
}
