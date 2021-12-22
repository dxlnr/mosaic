// use num::{bigint::BigInt, rational::Ratio};
use serde::{Deserialize, Serialize};
use std::slice::{Iter, IterMut};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub fn new(length: usize) -> Self {
        Model(vec![0.0; length])
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
    /// Elementwise addition some data to the ['Model'] object.
    pub fn add(&mut self, data: &Vec<f64>) {
        self.0 = self
            .iter()
            .zip(data)
            .map(|(s, x)| s + x)
            .collect::<Vec<_>>()
            .to_vec();
    }
    /// Elementwise averaging of ['Model'] object depending on the number of participants.
    pub fn avg(&mut self, participants: &u32) {
        self.0 = self
            .iter()
            .map(|x| x / *participants as f64)
            .collect::<Vec<_>>()
            .to_vec();
    }
}

#[derive(Debug)]
/// Data type that defines how byte stream of model is converted.
pub enum DataType {
    F32,
    F64,
}
