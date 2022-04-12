use rayon::prelude::*;
use rug::{ops::Pow, Float};
use std::ops::{Add, Div, Mul, Sub};

use crate::core::{
    aggregator::{features::Features, AggregationParams},
    model::Model,
};

pub trait FedOpt {
    /// Computes the delta_t parameter.
    fn get_delta_t(&mut self, mut features: Features, upd_model: &Model) -> Model {
        if features.global.is_empty() {
            features.global = Model::zeros(&upd_model.len());
        }
        let delta = upd_model
            .0
            .par_iter()
            .zip(features.global.0.par_iter())
            .map(|(x_ki, x_ti)| x_ki.clone().sub(x_ti))
            .collect::<Vec<_>>();
        Model(delta)
    }

    /// Computes the m_t term.
    fn get_m_t(
        &mut self,
        params: &AggregationParams,
        mut features: Features,
        delta_t: &Model,
    ) -> Model {
        if features.m_t.is_empty() {
            features.m_t = Model::zeros(&delta_t.len());
        }
        let m_t_upd = delta_t
            .0
            .par_iter()
            .zip(features.m_t.0.par_iter())
            .map(|(delta_ti, m_ti)| {
                m_ti.mul(params.get_beta_1())
                    .add(delta_ti.mul(Float::with_val(53, 1).sub(params.get_beta_1())))
            })
            .collect::<Vec<_>>();
        Model(m_t_upd)
    }

    /// Computes the v_t term. This is specific to different aggregation strategies.
    fn get_v_t(&mut self, delta_t: &Model) -> Model;

    /// Computes adjustment term. eta * ( m_t / (sqrt(v_t) + tau) )
    fn get_adjustment(&mut self, params: &AggregationParams, m_t: &Model, v_t: &Model) -> Model {
        let factor = m_t
            .0
            .par_iter()
            .zip(v_t.0.par_iter())
            .map(|(m_ti, v_ti)| {
                params
                    .get_eta()
                    .mul(m_ti.div(v_ti.clone().pow(-2_i32).add(params.get_tau())))
            })
            .collect::<Vec<_>>();
        Model(factor)
    }

    /// Computes new aggregated model.
    fn adjust(
        &mut self,
        params: &AggregationParams,
        upd_model: &Model,
        m_t: &Model,
        v_t: &Model,
    ) -> Model {
        let adj = self.get_adjustment(params, m_t, v_t);
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
    use crate::core::aggregator::{features::Features, Aggregator, Baseline, FedAdam};
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
            .avg(&aggr.features.locals, &aggr.features.prep_stakes()).unwrap();
        let delta_t = aggr.get_delta_t(aggr.features.clone(), &upd_model);

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

        let delta_t = aggr.get_delta_t(aggr.features.clone(), &upd_model);
        let m_t_upd = aggr.get_m_t(&aggr.base.params.clone(), aggr.features.clone(), &delta_t);

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

        let delta_t = aggr.get_delta_t(aggr.features.clone(), &upd_model);
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

        let adjust_fac = aggr.get_adjustment(&aggr.base.params.clone(), &m_t_upd, &v_t_upd);

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
    //     let final_model = aggr.adjust(&m_t_upd, &v_t_upd);

    //     assert_eq!(
    //         final_model,
    //         Model(vec![
    //             Float::with_val(53, (0.1),
    //             Float::with_val(53, (0.1),
    //             Float::with_val(53, (0.1),
    //             Float::with_val(53, (0.1),
    //         ])
    //     );
    // }
}
