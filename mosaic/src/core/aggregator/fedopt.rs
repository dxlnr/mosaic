use rayon::prelude::*;
use rug::{Float, ops::Pow};
use std::ops::{Add, Div, Mul, Sub};

use crate::core::{
    aggregator::{AggregationParams, features::Features},
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
    fn get_m_t(&mut self, params: &AggregationParams, mut features: Features, delta_t: &Model) -> Model {
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
    fn adjust(&mut self, params: &AggregationParams, upd_model: &Model, m_t: &Model, v_t: &Model) -> Model {
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