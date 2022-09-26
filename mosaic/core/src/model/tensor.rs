//! Tensor library for Mosaic.
//!
use derive_more::{Display, From, Index, IndexMut, Into};
use rug::Float;
use std::{fmt::Debug, slice::Iter};
// use serde::{Deserialize, Serialize};
use protobuf::ProtobufEnum;

use crate::protos;

/// Derive conversion [`From`] and [`Into`] trait as macro for DataType.
///
macro_rules! enum_derive {
    (
        #[repr($T: ident)]
        $( #[$meta: meta] )*
        $vis: vis enum $Name: ident {
            $(
                $Variant: ident = $value: expr
            ),*
            $( , )?
        }
    ) => {
        #[repr($T)]
        $( #[$meta] )*
        $vis enum $Name {
            $(
                $Variant = $value
            ),*
        }
        impl std::convert::From<$T> for $Name {
            fn from(value: $T) -> $Name {
                match value {
                    $(
                        $value => $Name::$Variant,
                    )*
                    _ => $Name::DTInvalid,
                }
            }
        }
        impl std::convert::Into<$T> for $Name {
            fn into(self) -> $T {
                match self {
                    $(
                        $Name::$Variant => $value,
                    )*
                }
            }
        }
    }
}

enum_derive! {
    #[repr(i32)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, Display)]
    pub enum DataType {
        DTInvalid = 0,

        DTf16 = 1,
        DTf32 = 2,
        DTf64 = 3,

        DTint8 = 4,
        DTint16 = 5,
        DTint32 = 6,
        DTint64 = 7,

        DTuint8 = 8,
        DTuint16 = 9,
        DTuint32 = 10,
        DTuint64 = 11,

        DTString = 12,
    }
}

impl DataType {
    /// Creates new DataType from `i32`.
    ///
    fn new(variant: i32) -> Self {
        Self::from(variant)
    }

    /// Converts [`DataType`] into proto DataType.
    ///
    fn into_proto(self) -> protos::dtype::DataType {
        if let Some(d) = protos::dtype::DataType::from_i32(self.into()) {
            d
        } else {
            // This is unfortunate, but the protobuf crate doesn't support unrecognized enum values.
            panic!("Unable to convert {} to a protobuf DataType", self);
        }
    }
    /// Converts proto DataType into [`DataType`].
    ///
    fn from_proto(proto: protos::dtype::DataType) -> Self {
        Self::from(proto.value() as i32)
    }
}

/// A [`TensorShape`] is the shape of a tensor. A TensorShape may be an unknown rank, or it may
/// have a known rank with each dimension being known or unknown.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Hash, Clone, Default)]
pub struct TensorShape(Option<Vec<Option<i32>>>);

impl TensorShape {
    /// Creates a new [`TensorShape`].
    pub fn new(s: Option<Vec<Option<i32>>>) -> Self {
        Self(s)
    }

    fn init(s: Vec<i32>) -> Self {
        let dims = s.iter().map(|s| Some(*s)).collect::<Vec<_>>();
        Self(Some(dims))
    }
    /// Returns the number of dimensions if known, or None if unknown.
    pub fn dims(&self) -> Option<usize> {
        match *self {
            TensorShape(None) => None,
            TensorShape(Some(ref v)) => Some(v.len()),
        }
    }

    /// Converts [`Tensorshape`] into proto message shape.
    fn into_proto(self) -> protos::tensor_shape::TensorShape {
        match self.0 {
            None => {
                let mut shape = protos::tensor_shape::TensorShape::new();
                shape.set_unknown_rank(true);
                shape
            }
            Some(v) => {
                let mut shape = protos::tensor_shape::TensorShape::new();
                for in_dim in v {
                    shape.mut_dim().push({
                        let mut out_dim = protos::tensor_shape::TensorShape_Dim::new();
                        out_dim.set_size(in_dim.unwrap_or(-1));
                        out_dim
                    });
                }
                shape
            }
        }
    }
    /// Converts proto message shape into [`Tensorshape`].
    fn from_proto(proto: &protos::tensor_shape::TensorShape) -> Self {
        TensorShape(if proto.get_unknown_rank() {
            None
        } else {
            Some(
                proto
                    .get_dim()
                    .iter()
                    .map(|dim| {
                        if dim.get_size() == -1 {
                            None
                        } else {
                            Some(dim.get_size())
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, From, Index, IndexMut, Into)]
/// A numerical representation of the weights contained by a Machine Learning model.
///
/// The representation lays out each element of the tensor contiguously in memory.
pub struct TensorStorage(Vec<Float>);

impl Default for TensorStorage {
    fn default() -> Self {
        Self(Vec::new())
    }
}

/// Conversion into an [`Iterator`].
///
/// By implementing `IntoIterator`, [`Iterator::collect()`] method is enabled
/// which allows to create a collection from the contents of an iterator.
///
impl FromIterator<Float> for TensorStorage {
    fn from_iter<T: IntoIterator<Item = Float>>(iter: T) -> Self {
        let mut tstore = TensorStorage::default();

        for i in iter {
            tstore.0.push(i);
        }
        tstore
    }
}
/// An interface to convert a collection of primitive values into an iterator of numerical values.
///
/// This trait is used to convert primitive types ([`f32`], [`f64`], [`i32`], [`i64`]) into a
/// [`Model`], which has its own internal representation of the weights. The opposite trait is
/// [`IntoPrimitives`].
pub trait FromPrimitives<N: Debug>: Sized {
    /// Creates an iterator from primitive values that yields converted numerical values.
    ///
    /// # Panics
    ///
    /// Panics if `prec` is out of the allowed range.
    fn from_primitives<I: Iterator<Item = N>>(iter: I) -> Self;
}

impl FromPrimitives<f32> for TensorStorage {
    fn from_primitives<I>(iter: I) -> Self
    where
        I: Iterator<Item = f32>,
    {
        iter.map(|n| Float::with_val(53, n)).collect()
    }
}

impl FromPrimitives<f64> for TensorStorage {
    fn from_primitives<I>(iter: I) -> Self
    where
        I: Iterator<Item = f64>,
    {
        iter.map(|n| Float::with_val(53, n)).collect()
    }
}

/// Single Model [`Tensor`].
///
#[derive(Debug, Clone)]
pub struct Tensor {
    /// [`TensorStorage`]
    pub storage: TensorStorage,
    /// [`DataType`]
    pub dtype: DataType,
    /// [`TensorShape`]
    pub shape: TensorShape,
}

impl Tensor {
    pub fn new(storage: TensorStorage, dtype: DataType, shape: TensorShape) -> Self {
        Self {
            storage,
            dtype,
            shape,
        }
    }
    pub fn init(storage: TensorStorage, dtype: i32, shape: Vec<i32>) -> Self {
        Self {
            storage,
            dtype: DataType::new(dtype),
            shape: TensorShape::init(shape),
        }
    }
    /// Creates an iterator that yields references to the weights/parameters
    /// of this [`Tensor`].
    ///
    pub fn iter(&self) -> Iter<Float> {
        self.storage.0.iter()
    }
}
