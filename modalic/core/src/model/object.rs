// use derive_more::{Display, From, Index, IndexMut, Into};
use num::{
    bigint::BigInt,
    rational::Ratio,
};
use serde::{Deserialize, Serialize};

use crate::model::DataType;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
/// A [`ModelObject`] which represents a model and some attached meta data.
/// 
pub struct ModelObject {
    pub data_type: DataType,
    pub data: Vec<Ratio<BigInt>>,
}

impl ModelObject {
    /// Creates a new [`ModelObject`] from given data vector and [`DataType`].
    /// 
    pub fn new(data: Vec<Ratio<BigInt>>, data_type: DataType) -> Self {
        Self { data_type, data }
    }
    /// Creates a new empty [`ModelObject`] and [`DataType`].
    /// 
    pub fn empty(data_type: DataType) -> Self {
        Self {
            data: Vec::new(),
            data_type,
        }
    }
}