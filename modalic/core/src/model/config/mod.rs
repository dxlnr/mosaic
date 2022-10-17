//! Model meta & configuration parameters.
//!
//! See the [model module] documentation since this is a private module anyways.
//!
//! [model module]: crate::model
pub(crate) mod serialize;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InvalidModelConfigError {
    #[error("Model uses invalid data type.")]
    DataType,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
/// The original primitive data type of the numerical values to be masked.
pub enum DataType {
    /// Numbers of type f32.
    F32 = 0,
    /// Numbers of type f64.
    F64 = 1,
    /// Numbers of type i32.
    I32 = 2,
    /// Numbers of type i64.
    I64 = 3,
}

impl TryFrom<u8> for DataType {
    type Error = InvalidModelConfigError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            0 => Ok(DataType::F32),
            1 => Ok(DataType::F64),
            2 => Ok(DataType::I32),
            3 => Ok(DataType::I64),
            _ => Err(InvalidModelConfigError::DataType),
        }
    }
}

impl TryInto<u8> for DataType {
    type Error = InvalidModelConfigError;

    fn try_into(self) -> Result<u8, Self::Error> {
        match self {
            DataType::F32 => Ok(0),
            DataType::F64 => Ok(1),
            DataType::I32 => Ok(2),
            DataType::I64 => Ok(3),
        }
    }
}

impl DataType {
    pub(crate) fn bytes_per_number(&self) -> usize {
        match self {
            DataType::F32 => 4,
            DataType::F64 => 8,
            DataType::I32 => 4,
            DataType::I64 => 8,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// Meta data regarding the model.
///
/// This configuration is applied for masking, aggregation and unmasking of models.
pub struct ModelConfig {
    /// The original primitive data type of the numerical values to be masked.
    pub data_type: DataType,
}

impl ModelConfig {
    /// Returns the number of bytes needed for an element of a model object.
    ///
    /// # Panics
    /// Panics if the bytes per number can't be represented as usize.
    pub(crate) fn bytes_per_number(&self) -> usize {
        self.data_type.bytes_per_number()
    }
}
