//! # Model
//!
//! A [`Model`] is a collection of weights/parameters which are represented as finite numerical
//! values (i.e. rational numbers) of arbitrary precision. As such, a model in itself is not bound
//! to any particular primitive data type, but it can be created from those and converted back into
//! them.
//!
//! Currently, the primitive data types [`f32`], [`f64`], [`i32`] and [`i64`] are supported and
//! this might be extended in the future.
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
//!
pub(crate) mod config;
pub(crate) mod model;
pub(crate) mod object;
pub(crate) mod serialize;

pub use self::{
    config::{DataType, ModelConfig},
    model::{
        bytes_to_ratio, ratio_to_bytes, FromPrimitives, IntoPrimitives, Model, ModelCastError,
        PrimitiveCastError,
    },
    object::ModelObject,
};
