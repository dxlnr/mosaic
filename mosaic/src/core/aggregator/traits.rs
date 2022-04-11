use derive_more::Display;
use rayon::prelude::*;
use rug::Float;
use serde::{Deserialize, Serialize};
use std::{
    ops::{Add, Mul, Sub},
    str::FromStr,
};
use tracing::log::warn;

use crate::core::{
    aggregator::{features::Features, fedopt::FedOpt, Baseline},
    model::Model,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Display)]
/// The name of the aggregation scheme which determines the way the aggregation will work.
pub enum Scheme {
    #[display(fmt = "FedAvg")]
    FedAvg,
    #[display(fmt = "FedAdaGrad")]
    FedAdaGrad,
    #[display(fmt = "FedAdam")]
    FedAdam,
    #[display(fmt = "FedYogi")]
    FedYogi,
}

impl FromStr for Scheme {
    type Err = ();

    /// Returns a new [`Scheme`] from the given string,
    /// or an error if any strings are invalid.
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "FedAvg" => Ok(Scheme::FedAvg),
            "FedAdaGrad" => Ok(Scheme::FedAdaGrad),
            "FedAdam" => Ok(Scheme::FedAdam),
            "FedYogi" => Ok(Scheme::FedYogi),
            _ => {
                warn!("Aggregation strategy {:?} not valid. Please choose from [FedAvg, FedAdaGrad, FedAdam, FedYogi].", &s);
                Ok(Scheme::FedAvg)
            }
        }
    }
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

/// [`FedAvg`] algorithm based on
/// [McMahan et al. Communication-Efficient Learning of Deep Networks from Decentralized Data](https://arxiv.org/abs/1602.05629)
#[derive(Debug, Default)]
pub struct FedAvg;

impl Strategy for Aggregator<FedAvg> {
    const NAME: Scheme = Scheme::FedAvg;

    fn aggregate(&mut self) -> (Model, Model, Model) {
        warn!("Using FedAvg now");
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

/// [`FedAdam`]: A federated version of the adaptive optimizer FedOpt based on
/// [Reddi et al. Adaptive Federated Optimization](https://arxiv.org/pdf/2003.00295.pdf)
#[derive(Debug, Default)]
pub struct FedAdam;

impl Strategy for Aggregator<FedAdam> {
    const NAME: Scheme = Scheme::FedAdam;

    fn aggregate(&mut self) -> (Model, Model, Model) {
        warn!("Using FedAdam now");
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

/// [`FedAdaGrad`]: A federated version of the adaptive optimizer FedOpt based on
/// [Reddi et al. Adaptive Federated Optimization](https://arxiv.org/pdf/2003.00295.pdf)
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
    fn get_v_t(&mut self, delta_t: &Model) -> Model {
        if self.features.v_t.is_empty() {
            self.features.v_t = Model::zeros(&delta_t.len());
        }
        let v_t_upd = delta_t
            .0
            .par_iter()
            .zip(self.features.v_t.0.par_iter())
            .map(|(delta_ti, v_ti)| v_ti.clone().add(delta_ti.clone().mul(delta_ti)))
            .collect::<Vec<_>>();
        Model(v_t_upd)
    }
}

/// [`FedYogi`]: A federated version of the adaptive optimizer FedOpt based on
/// [Reddi et al. Adaptive Federated Optimization](https://arxiv.org/pdf/2003.00295.pdf)
#[derive(Debug, Default)]
pub struct FedYogi;

impl Aggregator<FedYogi> {
    /// Creates a new [`Aggregator`] which uses [`FedYogi`] implementation as aggregation strategy.
    ///
    /// # Example
    ///
    /// Aggregator::<FedYogi>::new(Baseline::default(), features)
    ///
    pub fn new(base: Baseline, features: Features) -> Self {
        Self {
            private: FedYogi,
            base,
            features,
        }
    }
}

impl Strategy for Aggregator<FedYogi> {
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

impl FedOpt for Aggregator<FedYogi> {
    fn get_v_t(&mut self, delta_t: &Model) -> Model {
        if self.features.v_t.is_empty() {
            self.features.v_t = Model::zeros(&delta_t.len());
        }
        let v_t_upd = delta_t
            .0
            .par_iter()
            .zip(self.features.v_t.0.par_iter())
            .map(|(delta_ti, v_ti)| {
                v_ti.clone().sub(
                    (Float::with_val(53, 1).sub(self.base.params.get_beta_2()))
                        .mul(delta_ti.clone().mul(delta_ti))
                        .mul((v_ti.sub(delta_ti.clone().mul(delta_ti))).signum()),
                )
            })
            .collect::<Vec<_>>();
        Model(v_t_upd)
    }
}
