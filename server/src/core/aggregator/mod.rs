pub mod features;
pub mod traits;

use self::traits::FedAdam;
use crate::core::model::Model;

pub enum Scheme {
    FedAvg,
    FedAdam,
}

/// Parameters that fascilitate the aggregation schema.
#[derive(Debug, Clone)]
pub struct AggregationParams {
    // pub delta_t: f64,
    // pub m_t: f64,
    // pub v_t: f64,
    /// Server-side learning rate. Defaults to 1e-1.
    pub eta: f64,
    /// Momentum parameter. Defaults to 0.9
    pub beta_1: f64,
    /// Second moment parameter. Defaults to 0.99.
    pub beta_2: f64,
    ///  Controls the algorithm's degree of adaptability. Defaults to 1e-9.
    pub tau_t: f64,
}

impl AggregationParams {
    pub fn new(eta: f64, beta_1: f64, beta_2: f64, tau_t: f64) -> Self {
        Self { eta, beta_1, beta_2, tau_t }
    }
}

impl Default for AggregationParams {
    fn default() -> Self {
        Self {
            eta: 1e-1,
            beta_1: 0.9,
            beta_2: 0.99,
            tau_t: 1e-9
        }
    }
}

#[derive(Debug, Clone)]
pub struct Aggregator<S> {
    pub strategy: S,
}


impl<S> FedAdam for Aggregator<S> 
where 
    S: FedAdam,
{
    fn adapt(&mut self) -> Model {
        todo!()
    }
}

