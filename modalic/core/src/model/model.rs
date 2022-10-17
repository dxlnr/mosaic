//! Model representation and conversion.
//!
//! See the [module] documentation since this is a private module anyways.
//!
//! [model module]: crate::model
use std::{
    fmt::Debug,
    iter::{FromIterator, IntoIterator},
    slice::{Iter, IterMut},
};

use derive_more::{Display, From, Index, IndexMut, Into};
use num::{
    bigint::BigInt,
    clamp,
    rational::Ratio,
    traits::{float::FloatCore, identities::Zero, ToPrimitive},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::model::DataType;

#[derive(
    Debug, Clone, PartialEq, Eq, Hash, From, Index, IndexMut, Into, Serialize, Deserialize,
)]
/// A numerical representation of a machine learning model.
pub struct Model(pub Vec<Ratio<BigInt>>);

impl std::convert::AsRef<Model> for Model {
    fn as_ref(&self) -> &Model {
        self
    }
}

#[allow(clippy::len_without_is_empty)]
impl Model {
    /// Gets the number of weights/parameters of this model.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Creates an iterator that yields references to the weights/parameters of this model.
    pub fn iter(&self) -> Iter<Ratio<BigInt>> {
        self.0.iter()
    }

    /// Creates an iterator that yields mutable references to the weights/parameters of this model.
    pub fn iter_mut(&mut self) -> IterMut<Ratio<BigInt>> {
        self.0.iter_mut()
    }
}

impl FromIterator<Ratio<BigInt>> for Model {
    fn from_iter<I: IntoIterator<Item = Ratio<BigInt>>>(iter: I) -> Self {
        let data: Vec<Ratio<BigInt>> = iter.into_iter().collect();
        Model(data)
    }
}

impl IntoIterator for Model {
    type Item = Ratio<BigInt>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Display)]
/// A primitive data type as a target for model conversion.
pub(crate) enum PrimitiveType {
    F32,
    F64,
    I32,
    I64,
}

#[derive(Error, Debug)]
#[error("Could not convert weight {weight} to primitive type {target}")]
/// Errors related to model conversion into primitives.
pub struct ModelCastError {
    weight: Ratio<BigInt>,
    target: PrimitiveType,
}

#[derive(Clone, Error, Debug)]
#[error("Could not convert primitive type {0:?} to weight")]
/// Errors related to weight conversion from primitives.
pub struct PrimitiveCastError<P: Debug>(pub(crate) P);

/// An interface to convert a collection of numerical values into an iterator of primitive values.
///
/// This trait is used to convert a [`Model`], which has its own internal representation of the
/// weights, into primitive types ([`f32`], [`f64`], [`i32`], [`i64`]). The opposite trait is
/// [`FromPrimitives`].
pub trait IntoPrimitives<P: 'static>: Sized {
    /// Creates an iterator from numerical values that yields converted primitive values.
    ///
    /// # Errors
    /// Yields an error for each numerical value that can't be converted into a primitive value.
    fn into_primitives(self) -> Box<dyn Iterator<Item = Result<P, ModelCastError>>>;

    /// Creates an iterator from numerical values that yields converted primitive values.
    ///
    /// # Errors
    /// Yields an error for each numerical value that can't be converted into a primitive value.
    fn to_primitives(&self) -> Box<dyn Iterator<Item = Result<P, ModelCastError>>>;

    /// Consume this model and into an iterator that yields `P` values.
    ///
    /// # Panics
    /// Panics if a numerical value can't be converted into a primitive value.
    fn into_primitives_unchecked(self) -> Box<dyn Iterator<Item = P>> {
        Box::new(
            self.into_primitives()
                .map(|res| res.expect("conversion to primitive type failed")),
        )
    }
}

/// An interface to convert a collection of primitive values into an iterator of numerical values.
///
/// This trait is used to convert primitive types ([`f32`], [`f64`], [`i32`], [`i64`]) into a
/// [`Model`], which has its own internal representation of the weights. The opposite trait is
/// [`IntoPrimitives`].
pub trait FromPrimitives<P: Debug>: Sized {
    /// Creates an iterator from primitive values that yields converted numerical values.
    ///
    /// # Errors
    /// Yields an error for the first encountered primitive value that can't be converted into a
    /// numerical value due to not being finite.
    fn from_primitives<I: Iterator<Item = P>>(iter: I) -> Result<Self, PrimitiveCastError<P>>;

    /// Creates an iterator from primitive values that yields converted numerical values.
    ///
    /// If a primitive value cannot be directly converted into a numerical value due to not being
    /// finite, it is clamped.
    fn from_primitives_bounded<I: Iterator<Item = P>>(iter: I) -> Self;
}

impl IntoPrimitives<i32> for Model {
    fn into_primitives(self) -> Box<dyn Iterator<Item = Result<i32, ModelCastError>>> {
        Box::new(self.0.into_iter().map(|i| {
            i.to_integer().to_i32().ok_or(ModelCastError {
                weight: i,
                target: PrimitiveType::I32,
            })
        }))
    }

    fn to_primitives(&self) -> Box<dyn Iterator<Item = Result<i32, ModelCastError>>> {
        let vec = self.0.clone();
        Box::new(vec.into_iter().map(|i| {
            i.to_integer().to_i32().ok_or(ModelCastError {
                weight: i,
                target: PrimitiveType::I32,
            })
        }))
    }
}

impl FromPrimitives<i32> for Model {
    fn from_primitives<I: Iterator<Item = i32>>(iter: I) -> Result<Self, PrimitiveCastError<i32>> {
        Ok(iter.map(|p| Ratio::from_integer(BigInt::from(p))).collect())
    }

    fn from_primitives_bounded<I: Iterator<Item = i32>>(iter: I) -> Self {
        Self::from_primitives(iter).unwrap()
    }
}

impl IntoPrimitives<i64> for Model {
    fn into_primitives(self) -> Box<dyn Iterator<Item = Result<i64, ModelCastError>>> {
        Box::new(self.0.into_iter().map(|i| {
            i.to_integer().to_i64().ok_or(ModelCastError {
                weight: i,
                target: PrimitiveType::I64,
            })
        }))
    }

    fn to_primitives(&self) -> Box<dyn Iterator<Item = Result<i64, ModelCastError>>> {
        let vec = self.0.clone();
        Box::new(vec.into_iter().map(|i| {
            i.to_integer().to_i64().ok_or(ModelCastError {
                weight: i,
                target: PrimitiveType::I64,
            })
        }))
    }
}

impl FromPrimitives<i64> for Model {
    fn from_primitives<I: Iterator<Item = i64>>(iter: I) -> Result<Self, PrimitiveCastError<i64>> {
        Ok(iter.map(|p| Ratio::from_integer(BigInt::from(p))).collect())
    }

    fn from_primitives_bounded<I: Iterator<Item = i64>>(iter: I) -> Self {
        Self::from_primitives(iter).unwrap()
    }
}

impl IntoPrimitives<f32> for Model {
    fn into_primitives(self) -> Box<dyn Iterator<Item = Result<f32, ModelCastError>>> {
        let iter = self.0.into_iter().map(|r| {
            ratio_to_float::<f32>(&r).ok_or(ModelCastError {
                weight: r,
                target: PrimitiveType::F32,
            })
        });
        Box::new(iter)
    }

    fn to_primitives(&self) -> Box<dyn Iterator<Item = Result<f32, ModelCastError>>> {
        let vec = self.0.clone();
        let iter = vec.into_iter().map(|r| {
            ratio_to_float::<f32>(&r).ok_or(ModelCastError {
                weight: r,
                target: PrimitiveType::F32,
            })
        });
        Box::new(iter)
    }
}

impl FromPrimitives<f32> for Model {
    fn from_primitives<I: Iterator<Item = f32>>(iter: I) -> Result<Self, PrimitiveCastError<f32>> {
        iter.map(|f| Ratio::from_float(f).ok_or(PrimitiveCastError(f)))
            .collect()
    }

    fn from_primitives_bounded<I: Iterator<Item = f32>>(iter: I) -> Self {
        iter.map(float_to_ratio_bounded::<f32>).collect()
    }
}

impl IntoPrimitives<f64> for Model {
    fn into_primitives(self) -> Box<dyn Iterator<Item = Result<f64, ModelCastError>>> {
        let iter = self.0.into_iter().map(|r| {
            ratio_to_float::<f64>(&r).ok_or(ModelCastError {
                weight: r,
                target: PrimitiveType::F64,
            })
        });
        Box::new(iter)
    }

    fn to_primitives(&self) -> Box<dyn Iterator<Item = Result<f64, ModelCastError>>> {
        let vec = self.0.clone();
        let iter = vec.into_iter().map(|r| {
            ratio_to_float::<f64>(&r).ok_or(ModelCastError {
                weight: r,
                target: PrimitiveType::F64,
            })
        });
        Box::new(iter)
    }
}

impl FromPrimitives<f64> for Model {
    fn from_primitives<I: Iterator<Item = f64>>(iter: I) -> Result<Self, PrimitiveCastError<f64>> {
        iter.map(|f| Ratio::from_float(f).ok_or(PrimitiveCastError(f)))
            .collect()
    }

    fn from_primitives_bounded<I: Iterator<Item = f64>>(iter: I) -> Self {
        iter.map(float_to_ratio_bounded::<f64>).collect()
    }
}

pub fn ratio_to_bytes(ratio: &Ratio<BigInt>, dtype: DataType) -> Vec<u8> {
    match dtype {
        DataType::F32 => ratio_to_float::<f32>(ratio)
            .unwrap_or(0.0)
            .to_le_bytes()
            .to_vec(),
        DataType::F64 => ratio_to_float::<f64>(ratio)
            .unwrap_or(0.0)
            .to_le_bytes()
            .to_vec(),
        DataType::I32 => ratio
            .to_integer()
            .to_i32()
            .unwrap_or(0)
            .to_le_bytes()
            .to_vec(),
        DataType::I64 => ratio
            .to_integer()
            .to_i64()
            .unwrap_or(0)
            .to_le_bytes()
            .to_vec(),
    }
}

pub fn bytes_to_ratio(bytes: &[u8], dtype: &DataType) -> Ratio<BigInt> {
    match dtype {
        DataType::F32 => {
            let chunk: [u8; 4] = bytes.try_into().unwrap();
            float_to_ratio_bounded(f32::from_be_bytes(chunk))
        }
        DataType::F64 => {
            let chunk: [u8; 8] = bytes.try_into().unwrap();
            float_to_ratio_bounded(f64::from_be_bytes(chunk))
        }
        DataType::I32 => unimplemented!(),
        DataType::I64 => unimplemented!(),
    }
}

/// Converts a numerical value into a primitive floating point value.
///
/// # Errors
/// Fails if the numerical value is not representable in the primitive data type.
pub fn ratio_to_float<F: FloatCore>(ratio: &Ratio<BigInt>) -> Option<F> {
    let min_value = Ratio::from_float(F::min_value()).unwrap();
    let max_value = Ratio::from_float(F::max_value()).unwrap();
    if ratio < &min_value || ratio > &max_value {
        return None;
    }

    let mut numer = ratio.numer().clone();
    let mut denom = ratio.denom().clone();
    // safe loop: terminates after at most bit-length of ratio iterations
    loop {
        if let (Some(n), Some(d)) = (F::from(numer.clone()), F::from(denom.clone())) {
            if n == F::zero() || d == F::zero() {
                break Some(F::zero());
            } else {
                let float = n / d;
                if float.is_finite() {
                    break Some(float);
                }
            }
        } else {
            numer >>= 1_usize;
            denom >>= 1_usize;
        }
    }
}

/// Converts the primitive floating point value into a numerical value.
///
/// Maps positive/negative infinity to max/min of the primitive data type and NaN to zero.
pub(crate) fn float_to_ratio_bounded<F: FloatCore>(f: F) -> Ratio<BigInt> {
    if f.is_nan() {
        Ratio::<BigInt>::zero()
    } else {
        let finite_f = clamp(f, F::min_value(), F::max_value());
        // safe unwrap: clamped weight is guaranteed to be finite
        Ratio::<BigInt>::from_float(finite_f).unwrap()
    }
}
