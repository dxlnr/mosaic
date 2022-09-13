//! Tensor library for Mosaic.
//! 
use derive_more::{Display, From, Index, IndexMut, Into};
use rug::Float;

// use crate::protos;

/// The single underlying [`DataType`] of the Tensor elements.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq, Display)]
pub enum DataType {
    DTf16 = 1,
    DTf32 = 2,
    DTf64 = 3,

    DIint8 = 4,
    DTint16 = 5,
    DTint32 = 6,
    DTint64 = 7,

    DTuintT8 = 8,
    DTuint16 = 9,
    DTuint32 = 10,
    DTuint64 = 11,

    DTString = 12,
}

impl Default for DataType {
    fn default() -> DataType {
        DataType::DTf32
    }
}

impl DataType {
    // We don't use Into, because we don't want this to be public API.
    fn into_proto(self) -> protos::DataType {
        if let Some(d) = protos::DataType::from_i32(self.to_int() as i32) {
            d
        } else {
            // This is unfortunate, but the protobuf crate doesn't support unrecognized enum values.
            panic!("Unable to convert {} to a protobuf DataType", self);
        }
    }

    // We don't use From, because we don't want this to be public API.
    fn from_proto(proto: protos::DataType) -> Self {
        Self::from_int(proto.value() as c_uint)
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
    fn into_proto(self) -> protos::TensorShape {
        match self.0 {
            None => {
                let mut shape = protos::TensorShape::new();
                shape.set_unknown_rank(true);
                shape
            }
            Some(v) => {
                let mut shape = protos::TensorShape::new();
                for in_dim in v {
                    shape.mut_dim().push({
                        let mut out_dim = protos::TensorShapeProto_Dim::new();
                        out_dim.set_size(in_dim.unwrap_or(-1));
                        out_dim
                    });
                }
                shape
            }
        }
    }
    /// Converts proto message shape into [`Tensorshape`].
    fn from_proto(proto: &protos::TensorShape) -> Self {
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

#[derive(Debug, Clone, PartialEq, From, Index, IndexMut, Into, Serialize, Deserialize)]
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