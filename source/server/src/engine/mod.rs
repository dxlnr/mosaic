/// Main module as it implements all the key functionality.
/// Aggregation of the global model, keeping track of the training state, publishing protocol events
/// and handling protocol errors.
use std::convert::Infallible;
pub mod message;
pub mod model;

pub enum Engine {
    Idle,
    // Collect,
    // Aggregate,
    // Shutdown,
}

// impl Engine {
//     pub async fn run(mut self) -> Option<()> {
//         todo!()
//         // loop {
//         // }
//     }
// }

pub struct EngineInitializer {}

impl EngineInitializer {
    /// Creates a new [`EngineInitializer`] which prepares the algortihm for aggregation.
    pub fn new() {
        todo!()
    }

    pub async fn init(self) -> Engine {
        todo!()
    }
    fn init_engine(self) {
        todo!()
    }
}
