//! ML Model representation.
//!
pub mod serde;
pub mod tensor;

use crate::model::tensor::Tensor;
use std::slice::Iter;

use crate::protos::mosaic::protos::TensorProto;

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
    /// Instantiate a [`Model`] via repeated [`TensorProto`] objects.
    /// 
    pub fn from_proto(proto_tensors: Vec<TensorProto>, model_version: u32) -> Self {
        let ts = proto_tensors.iter().map(|tp| Tensor::from_proto(tp)).collect();

        Self {tensors: ts, model_version}
    }
}
