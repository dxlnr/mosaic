// use derive_more::{Display, From, Index, IndexMut, Into};
use num::{bigint::BigInt, rational::Ratio};
use serde::{Deserialize, Serialize};

use crate::model::ModelConfig;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Serialize, Deserialize)]
/// A [`ModelObject`] which represents a model and some attached meta data.
///
pub struct ModelObject {
    pub config: ModelConfig,
    pub data: Vec<Ratio<BigInt>>,
}

impl ModelObject {
    /// Creates a new [`ModelObject`] from given data vector and [`ModelConfig`].
    ///
    pub fn new(data: Vec<Ratio<BigInt>>, config: ModelConfig) -> Self {
        Self { config, data }
    }
    /// Creates a new empty [`ModelObject`] and [`ModelConfig`].
    ///
    pub fn empty(config: ModelConfig) -> Self {
        Self {
            data: Vec::new(),
            config,
        }
    }
}
