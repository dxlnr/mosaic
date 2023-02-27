//! Masking, aggregation and unmasking of models.
//!
//! # Models
//! A [`Model`] is a collection of weights/parameters which are represented as finite numerical
//! values (i.e. rational numbers) of arbitrary precision. As such, a model in itself is not bound
//! to any particular primitive data type, but it can be created from those and converted back into
//! them.
//!
//! The primitive data types [`f32`], [`f64`], [`i32`] and [`i64`] are supported. 
//!
//! ```
//! # use mosaic_core::mask::{FromPrimitives, IntoPrimitives, Model};
//! let weights = vec![0_f32; 10];
//! let model = Model::from_primitives_bounded(weights.into_iter());
//! assert_eq!(
//!     model.into_primitives_unchecked().collect::<Vec<f32>>(),
//!     vec![0_f32; 10],
//! );
//! ```
pub(crate) mod config;
pub(crate) mod masking;
pub(crate) mod object;
pub(crate) mod scalar;
pub(crate) mod seed;

pub use self::{
    config::{
        serialization::MaskConfigBuffer,
        BoundType,
        GroupType,
        InvalidMaskConfigError,
        MaskConfig,
        MaskConfigPair,
        ModelType,
    },
    masking::{Aggregation, AggregationError, Masker, UnmaskingError},
    object::{
        serialization::vect::MaskVectBuffer,
        InvalidMaskObjectError,
        MaskObject,
        MaskUnit,
        MaskVect,
    },
    scalar::{FromPrimitive, IntoPrimitive, Scalar, ScalarCastError},
    seed::{EncryptedMaskSeed, MaskSeed},
};