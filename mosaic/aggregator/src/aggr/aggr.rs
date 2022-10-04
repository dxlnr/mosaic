use super::buffer::FedBuffer;

#[derive(Clone, Debug)]
pub struct Aggregator {
    /// Current progress towards an aggregation goal.
    pub round_id: u32,
    /// Hyperparameter comprised in [`AggrParams`].
    pub params: AggrParams,
    /// [`FedBuffer`]
    pub buffer: FedBuffer,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            round_id: 0,
            params: AggrParams::default(),
            buffer: FedBuffer::default(),
        }
    }
}

/// Parameters necessary for performing an aggregation schema.
#[derive(Debug, Clone)]
pub struct AggrParams {
    /// Server-side learning rate. Defaults to 1e-1.
    pub eta: f64,
    /// Client updates are stored in a buffer. A server update only takes place
    /// once K client updates are in the buffer, where K is the size of the buffer.
    ///
    /// According to [Nguyen et al. 2021](https://arxiv.org/abs/2106.06639) k = 10 seems to be
    /// a good fit that needs no further tuning.
    pub k: u32,
}

impl AggrParams {
    /// Creates new [`AggrParams`] which allows altering the default parameters.
    pub fn new(eta: f64, k: u32) -> Self {
        Self { eta, k }
    }
}

impl Default for AggrParams {
    fn default() -> Self {
        Self { eta: 1e-1, k: 10 }
    }
}