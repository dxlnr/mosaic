use derive_more::Display;
use rayon::prelude::*;
use rug::ops::Pow;
use rug::Float;
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
/// from Decentralized Data (https://arxiv.org/abs/1602.05629)
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

    fn aggregate(&mut self) -> (Model, Model, Model) {
        let upd_model = self
            .base
            .avg(&self.features.locals, &self.features.prep_stakes());

        let delta_t = self.get_delta_t(&upd_model);
        let m_t_upd = self.get_m_t(&delta_t);
        let v_t_upd = self.get_v_t(&delta_t);
        // let global = self.adjust(&m_t_upd, &v_t_upd);
        let global = self.adjust(&upd_model, &m_t_upd, &v_t_upd);

        (global, m_t_upd, v_t_upd)
    }

    fn set_feat(&mut self, features: Features) {
        self.features = features;
    }
}

impl Aggregator<FedAdam> {
    /// Creates a new FedAdam implementation.
    pub fn new(base: Baseline, features: Features) -> Self {
        Self {
            private: FedAdam,
            base,
            features,
        }
    }
    /// Computes the delta_t parameter from the referenced paper.
    fn get_delta_t(&mut self, upd_model: &Model) -> Model {
        if self.features.global.is_empty() {
            self.features.global = Model::zeros(&upd_model.len());
        }
        let delta = upd_model
            .0
            .par_iter()
            .zip(self.features.global.0.par_iter())
            .map(|(x_ki, x_ti)| x_ki.clone().sub(x_ti))
            .collect::<Vec<_>>();
        Model(delta)
    }
    /// Computes the m_t term.
    fn get_m_t(&mut self, delta_t: &Model) -> Model {
        if self.features.m_t.is_empty() {
            self.features.m_t = Model::zeros(&delta_t.len());
        }
        let m_t_upd = delta_t
            .0
            .par_iter()
            .zip(self.features.m_t.0.par_iter())
            .map(|(delta_ti, m_ti)| {
                m_ti.mul(self.base.params.get_beta_1())
                    .add(delta_ti.mul(Float::with_val(53, 1).sub(self.base.params.get_beta_1())))
            })
            .collect::<Vec<_>>();
        Model(m_t_upd)
    }
    /// Computes the v_t term for FedAdam specifically.
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
    /// Computes adjustment term. eta * ( m_t / (sqrt(v_t) + tau) )
    fn get_adjustment(&mut self, m_t: &Model, v_t: &Model) -> Model {
        let factor = m_t
            .0
            .par_iter()
            .zip(v_t.0.par_iter())
            .map(|(m_ti, v_ti)| {
                self.base
                    .params
                    .get_eta()
                    .mul(m_ti.div(v_ti.clone().pow(-2_i32).add(self.base.params.get_tau())))
            })
            .collect::<Vec<_>>();
        Model(factor)
    }
    /// Computes new aggregated model.
    fn adjust(&mut self, upd_model: &Model, m_t: &Model, v_t: &Model) -> Model {
        let adj = self.get_adjustment(m_t, v_t);
        let res = upd_model
            .0
            .par_iter()
            .zip(adj.0.par_iter())
            .map(|(x_ti, x_ai)| x_ti.clone().add(x_ai))
            .collect::<Vec<_>>();
        Model(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::aggregator::features::Features;
    use rug::Float;

    #[test]
    fn test_fedadam_aggregation() {
        let m1 = Model(vec![
            Float::with_val(53, 20.0),
            Float::with_val(53, 20.0),
            Float::with_val(53, 6.0),
            Float::with_val(53, 6.0),
        ]);
        let m2 = Model(vec![
            Float::with_val(53, 20.0),
            Float::with_val(53, 20.0),
            Float::with_val(53, 2.0),
            Float::with_val(53, 2.0),
        ]);
        let m3 = Model(vec![
            Float::with_val(53, 4.0),
            Float::with_val(53, 4.0),
            Float::with_val(53, 20.0),
            Float::with_val(53, 20.0),
        ]);
        let m4 = Model(vec![
            Float::with_val(53, 4.0),
            Float::with_val(53, 4.0),
            Float::with_val(53, 20.0),
            Float::with_val(53, 20.0),
        ]);

        let model_list = vec![m1, m2, m3, m4];
        let stakes = vec![1, 1, 1, 1];
        let mut feats = Features::new(model_list, stakes);

        feats.global = Model(vec![
            Float::with_val(53, 2.0),
            Float::with_val(53, 2.0),
            Float::with_val(53, 2.0),
            Float::with_val(53, 2.0),
        ]);

        let mut aggr = Aggregator::<FedAdam>::new(Baseline::default(), feats);

        let upd_model = aggr
            .base
            .avg(&aggr.features.locals, &aggr.features.prep_stakes());
        let delta_t = aggr.get_delta_t(&upd_model);

        assert_eq!(
            delta_t,
            Model(vec![
                Float::with_val(53, 10.0),
                Float::with_val(53, 10.0),
                Float::with_val(53, 10.0),
                Float::with_val(53, 10.0),
            ])
        );
    }

    #[test]
    fn test_get_m_t() {
        let feats = Features {
            global: Model(vec![
                Float::with_val(53, 2.0),
                Float::with_val(53, 2.0),
                Float::with_val(53, 2.0),
                Float::with_val(53, 2.0),
            ]),
            ..Default::default()
        };

        let upd_model = Model(vec![
            Float::with_val(53, 12.0),
            Float::with_val(53, 12.0),
            Float::with_val(53, 12.0),
            Float::with_val(53, 12.0),
        ]);

        let mut aggr = Aggregator::<FedAdam>::new(Baseline::default(), feats);

        let delta_t = aggr.get_delta_t(&upd_model);
        let m_t_upd = aggr.get_m_t(&delta_t);

        assert_eq!(
            m_t_upd,
            Model(vec![
                Float::with_val(53, 1.0),
                Float::with_val(53, 1.0),
                Float::with_val(53, 1.0),
                Float::with_val(53, 1.0),
            ])
        );
    }

    #[test]
    fn test_get_v_t() {
        let feats = Features {
            global: Model(vec![
                Float::with_val(53, 2.0),
                Float::with_val(53, 2.0),
                Float::with_val(53, 2.0),
                Float::with_val(53, 2.0),
            ]),
            ..Default::default()
        };

        let upd_model = Model(vec![
            Float::with_val(53, 12.0),
            Float::with_val(53, 12.0),
            Float::with_val(53, 12.0),
            Float::with_val(53, 12.0),
        ]);

        let mut aggr = Aggregator::<FedAdam>::new(Baseline::default(), feats);

        let delta_t = aggr.get_delta_t(&upd_model);
        let v_t_upd = aggr.get_v_t(&delta_t);

        assert_eq!(
            v_t_upd,
            Model(vec![
                Float::with_val(53, 1.0),
                Float::with_val(53, 1.0),
                Float::with_val(53, 1.0),
                Float::with_val(53, 1.0),
            ])
        );
    }

    #[test]
    fn test_adj_fac() {
        let feats = Features::default();
        let mut aggr = Aggregator::<FedAdam>::new(Baseline::default(), feats);

        let m_t_upd = Model(vec![
            Float::with_val(53, 1.0),
            Float::with_val(53, 1.0),
            Float::with_val(53, 1.0),
            Float::with_val(53, 1.0),
        ]);
        let v_t_upd = Model(vec![
            Float::with_val(53, 1.0),
            Float::with_val(53, 1.0),
            Float::with_val(53, 1.0),
            Float::with_val(53, 1.0),
        ]);

        let adjust_fac = aggr.get_adjustment(&m_t_upd, &v_t_upd);

        assert_eq!(
            adjust_fac,
            Model(vec![
                Float::with_val(53, 0.1),
                Float::with_val(53, 0.1),
                Float::with_val(53, 0.1),
                Float::with_val(53, 0.1),
            ])
        );
    }

    // #[test]
    // fn test_final_model() {
    //     let _final_model = aggr.adjust(&m_t_upd, &v_t_upd);

    //     assert_eq!(
    //         final_model,
    //         Model(vec![
    //             Float::with_val(53, (0.1_f32),
    //             Float::with_val(53, (0.1_f32),
    //             Float::with_val(53, (0.1_f32),
    //             Float::with_val(53, (0.1_f32),
    //         ])
    //     );
    // }
}
