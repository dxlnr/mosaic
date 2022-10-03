/// The buffer can be implemented by using a Trusted Execution Environment (TEE) or through
/// a cryptographic algorithm.
/// 
use std::collections::HashMap;

use mosaic_core::model::Model;
use super::counter::MessageCounter;

#[derive(Debug, Clone)]
pub struct FedBuffer<T> {
    /// [`MessageCounter`]
    counter: MessageCounter,
    /// Hashmap containing [`Model`] updates with associated training round.
    mmap: HashMap<u32, Vec<Model<T>>>,
}

impl<T> Default for FedBuffer<T> {
    /// Creates a new default [`MessageCounter`].
    fn default() -> Self {
        Self {
            counter: MessageCounter::default(),
            mmap: HashMap::new(),
        }
    }
}