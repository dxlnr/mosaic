//! Storage backends to manage the coordinator state.

pub mod noop;
#[cfg(feature = "redis")]
#[cfg_attr(docsrs, doc(cfg(feature = "redis")))]
pub mod redis;
