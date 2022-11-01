//! Storage backends for the coordinator.
//!
pub mod aggr_storage;
pub mod model_storage;
pub mod store;
pub mod traits;
pub mod trust_anchor;

pub use self::{
    store::Store,
    traits::{
        AggregatorStorage,
        LocalSeedDictAdd,
        LocalSeedDictAddError,
        MaskScoreIncr,
        MaskScoreIncrError,
        ModelStorage,
        Storage,
        StorageError,
        StorageResult,
        SumPartAdd,
        SumPartAddError,
        TrustAnchor,
    },
};
