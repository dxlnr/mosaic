use derive_more::Display;
use num::{bigint::BigInt, rational::Ratio, traits::One};
use std::ops::{Add, Div, Mul, Sub};

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
            .avg(&self.features.locals, &self.features.prep_stakes())
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
            .avg(&self.features.locals, &self.features.prep_stakes());
        let delta_t = self.get_delta_t(&upd_model);
        let m_t_upd = self.get_m_t(&delta_t);
        let v_t_upd = self.get_v_t(&delta_t);
        self.adjust(&m_t_upd, &v_t_upd)
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
            .map(|(x_ki, x_ti)| x_ki.sub(x_ti))
            .collect::<Vec<_>>();
        Model(delta)
    }
    /// Computes the m_t term.
    fn get_m_t(&mut self, delta_t: &Model) -> Model {
        let m_t_upd = delta_t
            .iter()
            .zip(self.features.m_t.iter())
            .map(|(delta_ti, m_ti)| {
                m_ti.mul(self.base.params.get_beta_1())
                    .add(delta_ti.mul(Ratio::<BigInt>::one().sub(self.base.params.get_beta_1())))
            })
            .collect::<Vec<_>>();
        Model(m_t_upd)
    }
    /// Computes the v_t term for FedAdam specifically.
    fn get_v_t(&mut self, delta_t: &Model) -> Model {
        let v_t_upd = delta_t
            .iter()
            .zip(self.features.v_t.iter())
            .map(|(delta_ti, v_ti)| {
                v_ti.mul(self.base.params.get_beta_2()).add(
                    (delta_ti.mul(delta_ti))
                        .mul(Ratio::<BigInt>::one().sub(self.base.params.get_beta_2())),
                )
            })
            .collect::<Vec<_>>();
        Model(v_t_upd)
    }
    /// Computes adjustment term. eta * ( m_t / (sqrt(v_t) + tau) )
    fn get_adjustment(&mut self, m_t: &Model, v_t: &Model) -> Model {
        let factor = m_t
            .iter()
            .zip(v_t.iter())
            .map(|(m_ti, v_ti)| {
                self.base
                    .params
                    .get_eta()
                    .mul(m_ti.div((v_ti.pow(-2)).add(self.base.params.get_tau())))
            })
            .collect::<Vec<_>>();
        Model(factor)
    }
    /// Computes new aggregated model.
    fn adjust(&mut self, m_t: &Model, v_t: &Model) -> Model {
        let adj = self.get_adjustment(m_t, v_t);
        let res = self
            .features
            .global
            .iter()
            .zip(adj.iter())
            .map(|(x_ti, x_ai)| x_ti.add(x_ai))
            .collect::<Vec<_>>();
        Model(res)
    }
}
