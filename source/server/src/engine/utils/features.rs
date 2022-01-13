use crate::engine::model::Model;
// use crate::message::Message;
use std::ops::Add;

#[derive(Debug)]
pub struct Features {
    // /// keeps msgs in cache that have been received by the clients.
    // pub msgs: Vec<Message>,
    pub locals: Vec<Model>,
    /// keeps track of the number of msgs received by the clients.
    pub factor: u32,
    // /// Will store the overall averaged vector of all messages.
    pub global: Model,
}

impl Features {
    /// Instantiates new ['Features'] object.
    pub fn new() -> Self {
        Features {
            locals: Vec::new(),
            factor: 0,
            global: Default::default(),
        }
    }
    /// Increment the factor which holds the number of received messages from previous.
    pub fn increment(&mut self, count: &u32) {
        self.factor += count;
    }

    /// Elementwise addition of (all) single msgs to the global field.
    pub fn add(&mut self) {
        // if self.factor != 0 {
        //     self.global.0 = self
        //         .global
        //         .0
        //         .iter()
        //         .map(|x| x * self.factor as f64)
        //         .collect::<Vec<_>>()
        //         .to_vec();
        // }
        self.locals
            .iter()
            .map(|r| {
                self.global.0 = self
                    .global
                    .0
                    .iter()
                    .zip(&r.0)
                    .map(|(l1, l2)| {
                        l1.iter()
                            .zip(l2)
                            .map(|(x1, x2)| x1.add(x2))
                            .collect::<Vec<_>>()
                            .to_vec()
                    })
                    .collect::<Vec<_>>()
                    .to_vec()
            })
            .collect::<Vec<_>>()
            .to_vec();
        // self.locals
        //     .iter()
        //     .map(|single_model| {
        //         single_model
        //             .0
        //             .iter()
        //             .map(|r| {
        //                 self.global.0 = self
        //                     .global
        //                     .0
        //                     .iter()
        //                     .zip(r)
        //                     .map(|(s, x)| *s.add(*x))
        //                     .collect::<Vec<_>>()
        //                     .to_vec()
        //             })
        //             .collect::<Vec<_>>()
        //             .to_vec();
        //     })
        //     .collect::<Vec<_>>()
        //     .to_vec();
    }
    // /// Averaging the summed global part of ['Features'].
    // pub fn avg(&mut self, participants: &u32, round_id: &u32) {
    //     self.global = self
    //         .global
    //         .iter()
    //         .map(|x| x / (*participants * *round_id) as f64)
    //         .collect::<Vec<_>>()
    //         .to_vec();
    // }
    //
    // /// Removing all messages from previous training round.
    // pub fn flush(&mut self) {
    //     self.msgs.clear();
    // }
}
