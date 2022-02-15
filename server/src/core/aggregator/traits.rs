use derive_more::Display;
use num::{bigint::BigInt, rational::Ratio, traits::One};
use std::ops::{Add, Mul, Sub};

use crate::core::{
    aggregator::{features::Features, Baseline},
    model::Model,
};

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
        self.base
            .avg(self.features.locals.clone(), self.features.prep_stakes())
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

/// FedAdam algorithm based on Reddi et al. ADAPTIVE FEDERATED OPTIMIZATION
/// (https://arxiv.org/pdf/2003.00295.pdf)
#[derive(Debug, Default)]
pub struct FedAdam;

impl Strategy for Aggregator<FedAdam> {
    const NAME: Scheme = Scheme::FedAdam;

    fn aggregate(&mut self) -> Model {
        let upd_model = self
            .base
            .avg(self.features.locals.clone(), self.features.prep_stakes());
        let delta_t = self.get_delta_t(&upd_model);
        let _m_t_upd = self.get_m_t(&delta_t);
        // let v_t_upd = self.get_v_t(&delta_t)
        self.get_v_t(&delta_t)

    }

    fn set_feat(&mut self, features: Features) {
        self.features = features;
    }
}
impl Aggregator<FedAdam> {
    /// Creates a new idle state.
    pub fn new(base: Baseline, features: Features) -> Self {
        Self {
            private: FedAdam,
            base,
            features,
        }
    }
    /// Compute the delta_t parameter from the referenced paper.
    fn get_delta_t(&mut self, upd_model: &Model) -> Model {
        let delta = upd_model
            .iter()
            .zip(self.features.global.iter())
            .map(|(x1, x2)| x1.sub(x2))
            .collect::<Vec<_>>();
        Model(delta)
    }

    fn get_m_t(&mut self, delta_t: &Model) -> Model {
        let m_t_upd = delta_t
            .iter()
            .zip(self.features.m_t.iter())
            .map(|(x1, x2)| {
                x2.mul(self.base.params.get_beta_1())
                    .add(x1.mul(Ratio::<BigInt>::one().sub(self.base.params.get_beta_1())))
            })
            .collect::<Vec<_>>();
        Model(m_t_upd)
    }
    
    fn get_v_t(&mut self, delta_t: &Model) -> Model {
        let v_t_upd = delta_t
            .iter()
            .zip(self.features.v_t.iter())
            .map(|(x1, x2)| {
                x2.mul(self.base.params.get_beta_2())
                    .add((x1.mul(x1)).mul(Ratio::<BigInt>::one().sub(self.base.params.get_beta_2())))
            })
            .collect::<Vec<_>>();
        Model(v_t_upd)
    }
}

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
