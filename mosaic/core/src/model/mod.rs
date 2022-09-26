//! ML Model representation.
//!
pub mod tensor;

use std::{fmt::Debug, slice::Iter};
use thiserror::Error;
// use rug::Float;
use crate::model::tensor::{FromPrimitives, Tensor, TensorStorage};

#[derive(Clone, Error, Debug)]
/// Errors that occur during casting to `rug::Float`.
pub struct CastingError<P: Debug>(pub(crate) P);

/// [`Model`] represents a Machine Learning model, adapted to FL.
/// 
#[derive(Default)]
pub struct Model {
    /// Actual ['Model'] content.
    pub tensors: Vec<Tensor>,
    /// Model version which returns the round_id in which the local model was trained
    /// or aggregated by the server.
    pub model_version: u32,
}

impl Model {
    pub fn new(tensors: Vec<Tensor>, model_version: u32) -> Self {
        Self {tensors, model_version}
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

// impl FromIterator<TensorStorage> for Vec<TensorStorage> {
//     fn from_iter<T: IntoIterator<Item = TensorStorage>>(iter: T) -> Self {
//         let mut tensors = Vec::new();

//         for i in iter {
//             tensors.push(i);
//         }
//         tensors
//     }
// }

/// An interface to convert a collection of primitive values into an iterator of numerical values.
///
/// This trait is used to convert primitive types ([`f32`], [`f64`], [`i32`], [`i64`]) into a
/// [`Model`], which has its own internal representation of the weights. The opposite trait is 
/// [`IntoPrimitives`].
pub trait FromTensors<N: Debug>: Sized {
    /// Creates an iterator from primitive values that yields converted numerical values.
    ///
    /// # Errors
    /// Yields an error for the first encountered primitive value that can't be converted into a
    /// numerical value due to not being finite.
    fn from_primitives<S: Iterator<Item = f32>>(storage: Vec<S>, dtype: Vec<i32>, shape: Vec<Vec<i32>>, model_version: u32) -> Self;
}

impl FromTensors<f32> for Model {
    fn from_primitives<S>(storage: Vec<S>, dtype: Vec<i32>, shape: Vec<Vec<i32>>, model_version: u32) -> Self 
    where
        S: Iterator<Item = f32>,
    {
        Model { tensors: storage.map(|t| Tensor::create(t, dtype, shape)).collect::<Vec<_>>(),
        model_version }
    }
}

// impl FromPrimitives<f32> for Model {
//     fn from_primitives<I: Iterator<Item = f32>>(iter: I) -> Self {
//         OK(iter.map(|t| Float::with_val(53, f))
//             .collect::<Vec<Tensor>>().to_vec())
//     }
// }