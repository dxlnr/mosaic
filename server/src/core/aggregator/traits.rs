// use num::{bigint::BigInt, rational::Ratio};
use derive_more::Display;

use crate::core::{aggregator::{features::Features, Baseline}, model::Model};

/// The name of the aggregation scheme.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Scheme {
    #[display(fmt = "FedAvg")]
    FedAvg,
    #[display(fmt = "FedAdam")]
    FedAdam,
}

pub trait Strategy {
    const NAME: Scheme;

    fn aggregate(&mut self) -> Model;
    fn set_feat(&mut self, features: Features);
}

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Aggregator<A> {
    /// Sets the aggregation strategy.
    pub(in crate::core::aggregator) private: A,
    /// Baseline Aggregator object.
    pub base: Baseline,
    /// feature set.
    pub features: Features,
}

/// FedAvg algorithm based on McMahan et al. Communication-Efficient Learning of Deep Networks
/// from Decentralized Data (https://arxiv.org/abs/1602.05629)
#[derive(Debug, Default)]
pub struct FedAvg;

impl Strategy for Aggregator<FedAvg> {
    const NAME: Scheme = Scheme::FedAvg;

    fn aggregate(&mut self) -> Model {
        self.base.avg(self.features.locals.clone(), self.features.prep_stakes())
    }

    fn set_feat(&mut self, features: Features) {
        self.features = features;
    }
}

impl Aggregator<FedAvg> {
    /// Creates a new idle state.
    pub fn new(features: Features) -> Self {
        Self {
            private: FedAvg,
            base: Baseline::default(),
            features,
        }
    }


}

// impl FedAdam for Baseline {
//     fn adapt(&mut self) -> Model {
//         todo!()
//     }
// }


// pub trait FedAvg
// where
//     Self: Clone + Send + Sync + 'static,
// {
//     fn aggregate(&mut self, features: Vec<Model>, stakes: Vec<Ratio<BigInt>>) -> Model;
// }

// /// FedAdam algorithm based on Reddi et al. ADAPTIVE FEDERATED OPTIMIZATION
// /// (https://arxiv.org/pdf/2003.00295.pdf)
// pub trait FedAdam
// where
//     Self: Clone + Send + Sync + 'static,
// {
//     fn adapt(&mut self) -> Model;
// }

// /// FedAdaGrad algorithm based on Reddi et al. ADAPTIVE FEDERATED OPTIMIZATION
// /// (https://arxiv.org/pdf/2003.00295.pdf)
// pub trait FedAdaGrad
// where
//     Self: Clone + Send + Sync + 'static,
// {
//     fn adapt(&mut self) -> Model;
// }
// /// FedYogi algorithm based on Reddi et al. ADAPTIVE FEDERATED OPTIMIZATION
// /// (https://arxiv.org/pdf/2003.00295.pdf)
// pub trait FedYogi
// where
//     Self: Clone + Send + Sync + 'static,
// {
//     fn adapt(&mut self) -> Model;
// }
