// use num::{bigint::BigInt, rational::Ratio};
use serde::{Deserialize, Serialize};
use std::slice::{Iter, IterMut};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// A representation of a machine learning model as vector object.
// pub struct Model(Vec<Ratio<BigInt>>);
pub struct Model(Vec<f64>);

impl std::convert::AsRef<Model> for Model {
    fn as_ref(&self) -> &Model {
        self
    }
}

impl Model {
    /// Instantiates a new empty model.
    pub fn new() -> Self {
        Model(Vec::new())
    }
    /// Returns the number of weights/parameters of a model.
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Creates an iterator that yields references to the weights/parameters of a model.
    pub fn iter(&self) -> Iter<f64> {
        self.0.iter()
    }

    /// Creates an iterator that yields mutable references to the weights/parameters of a model.
    pub fn iter_mut(&mut self) -> IterMut<f64> {
        self.0.iter_mut()
    }
}

#[derive(Debug)]
/// Data type that defines how byte stream of model is converted.
pub enum DataType {
    F32,
    F64,
}

// #[derive(Clone, Error, Debug)]
// #[error("Could not convert primitive type {0:?} to weight")]
// /// Errors related to weight conversion from bytes stream.
// pub struct CastingErrorFrom<T: Debug>(pub(crate) T);
//
// pub trait FromBytes {
//     fn from_bytes<I: Iterator<Item = T>>(iter: I) -> Result<Self, PrimitiveCastError<T>>;
// }
//
// impl FromBytes<f32> for Model {
//     fn from_bytes<I: Iterator<Item = f32>>(iter: I) -> Result<Self, CastingErrorFrom<f32>> {
//         iter.map(|f| BigEndian::read_f64(f).ok_or(CastingErrorFrom(f)))
//             .collect()
//     }
// }
