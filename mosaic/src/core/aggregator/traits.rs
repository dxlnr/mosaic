use derive_more::Display;
use rayon::prelude::*;
use rug::{ops::Pow, Float};
use std::ops::{Add, Div, Mul, Sub};

use crate::core::{
    aggregator::{features::Features, fedopt::FedOpt, Baseline},
    model::Model,
};

// use super::features;

/// The name of the aggregation scheme.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Scheme {
    #[display(fmt = "FedAvg")]
    FedAvg,
    #[display(fmt = "FedAdaGrad")]
    FedAdaGrad,
    #[display(fmt = "FedAdam")]
    FedAdam,
}

pub trait Strategy {
    const NAME: Scheme;
    /// Implementation of the aggregation algorithm based on the given strategy.
    fn aggregate(&mut self) -> (Model, Model, Model);
    /// Setting the features for each aggregation round.
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
/// from Decentralized Data `<https://arxiv.org/abs/1602.05629>`
#[derive(Debug, Default)]
pub struct FedAvg;

impl Strategy for Aggregator<FedAvg> {
    const NAME: Scheme = Scheme::FedAvg;

    fn aggregate(&mut self) -> (Model, Model, Model) {
        let global = self
            .base
            .avg(&self.features.locals, &self.features.prep_stakes());
        (global, Model::default(), Model::default())
    }
    fn set_feat(&mut self, features: Features) {
        self.features = features;
    }
}

impl Aggregator<FedAvg> {
    /// Creates a new [`Aggregator`] which uses [`FedAvg`] as baseline aggregation strategy.
    ///
    /// # Example
    ///
    /// Aggregator::<FedAvg>::new(features)
    ///
    pub fn new(features: Features) -> Self {
        Self {
            private: FedAvg,
            base: Baseline::default(),
            features,
        }
    }
}

/// FedAdam algorithm based on Reddi et al. ADAPTIVE FEDERATED OPTIMIZATION
/// `<https://arxiv.org/pdf/2003.00295.pdf>`
#[derive(Debug, Default)]
pub struct FedAdam;

impl Strategy for Aggregator<FedAdam> {
    const NAME: Scheme = Scheme::FedAdam;

    fn aggregate(&mut self) -> (Model, Model, Model) {
        let upd_model = self
            .base
            .avg(&self.features.locals, &self.features.prep_stakes());

        let delta_t = self.get_delta_t(self.features.clone(), &upd_model);
        let m_t_upd = self.get_m_t(&self.base.params.clone(), self.features.clone(), &delta_t);
        let v_t_upd = self.get_v_t(&delta_t);
        let global = self.adjust(&self.base.params.clone(), &upd_model, &m_t_upd, &v_t_upd);

        (global, m_t_upd, v_t_upd)
    }

    fn set_feat(&mut self, features: Features) {
        self.features = features;
    }
}

impl Aggregator<FedAdam> {
    /// Creates a new [`Aggregator`] which uses [`FedAdam`] implementation as aggregation strategy.
    ///
    /// # Example
    ///
    /// Aggregator::<FedAdam>::new(Baseline::default(), features)
    ///
    pub fn new(base: Baseline, features: Features) -> Self {
        Self {
            private: FedAdam,
            base,
            features,
        }
    }
}

impl FedOpt for Aggregator<FedAdam> {
    fn get_v_t(&mut self, delta_t: &Model) -> Model {
        if self.features.v_t.is_empty() {
            self.features.v_t = Model::zeros(&delta_t.len());
        }
        let v_t_upd = delta_t
            .0
            .par_iter()
            .zip(self.features.v_t.0.par_iter())
            .map(|(delta_ti, v_ti)| {
                v_ti.mul(self.base.params.get_beta_2()).add(
                    (delta_ti.clone().mul(delta_ti))
                        .mul(Float::with_val(53, 1).sub(self.base.params.get_beta_2())),
                )
            })
            .collect::<Vec<_>>();
        Model(v_t_upd)
    }
}

/// [`FedAdaGrad`]: A federated version of the adaptive optimizer FedOpt
/// based on Reddi et al. ADAPTIVE FEDERATED OPTIMIZATION
#[derive(Debug, Default)]
pub struct FedAdaGrad;

impl Aggregator<FedAdaGrad> {
    /// Creates a new [`Aggregator`] which uses [`FedAdaGrad`] implementation as aggregation strategy.
    ///
    /// # Example
    ///
    /// Aggregator::<FedAdaGrad>::new(Baseline::default(), features)
    ///
    pub fn new(base: Baseline, features: Features) -> Self {
        Self {
            private: FedAdaGrad,
            base,
            features,
        }
    }
}

impl Strategy for Aggregator<FedAdaGrad> {
    const NAME: Scheme = Scheme::FedAdaGrad;

    fn aggregate(&mut self) -> (Model, Model, Model) {
        let upd_model = self
            .base
            .avg(&self.features.locals, &self.features.prep_stakes());

        let delta_t = self.get_delta_t(self.features.clone(), &upd_model);
        let m_t_upd = self.get_m_t(&self.base.params.clone(), self.features.clone(), &delta_t);
        let v_t_upd = self.get_v_t(&delta_t);
        let global = self.adjust(&self.base.params.clone(), &upd_model, &m_t_upd, &v_t_upd);

        (global, m_t_upd, v_t_upd)
    }

    fn set_feat(&mut self, features: Features) {
        self.features = features;
    }
}

impl FedOpt for Aggregator<FedAdaGrad> {
    fn get_v_t(&mut self, _delta_t: &Model) -> Model {
        todo!()
    }
}
