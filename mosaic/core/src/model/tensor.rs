//! Tensor library for Mosaic.
//!
use derive_more::{Display, From, Index, IndexMut, Into};
use rug::Float;
// use serde::{Deserialize, Serialize};
use protobuf::ProtobufEnum;

use crate::protos;

/// Derive Convertion function as macro for DataType.
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
    /// Converts [`DataType`] into proto DataType.
    fn into_proto(self) -> protos::dtype::DataType {
        if let Some(d) = protos::dtype::DataType::from_i32(self.into()) {
            d
        } else {
            // This is unfortunate, but the protobuf crate doesn't support unrecognized enum values.
            panic!("Unable to convert {} to a protobuf DataType", self);
        }
    }
    /// Converts proto DataType into [`DataType`].
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

pub struct Tensor {
    pub storage: TensorStorage,
    pub dtype: DataType,
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
}
