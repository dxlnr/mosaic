use crate::engine::model::Model;
use crate::message::Message;

#[derive(Debug)]
pub struct Features {
    // /// keeps msgs in cache that have been received by the clients.
    // pub msgs: Vec<Message>,
    pub locals: Vec<Model>,
    /// keeps track of the number of msgs received by the clients.
    pub factor: u32,
    // /// Will store the overall averaged vector of all messages.
    // pub global: Model,
}

impl Features {
    /// Instantiates new ['Features'] object.
    pub fn new() -> Self {
        Features {
            locals: Vec::new(),
            factor: 0,
        }
    }
    /// Increment the factor which holds the number of received messages from previous.
    pub fn increment(&mut self, count: &u32) {
        self.factor += count;
    }

    // /// Elementwise addition of (all) single msgs to the global field.
    // pub fn add(&mut self) {
    //     if self.factor != 0 {
    //         self.global = self
    //             .global
    //             .iter()
    //             .map(|x| x * self.factor as f64)
    //             .collect::<Vec<_>>()
    //             .to_vec();
    //     }
    //     self.msgs
    //         .iter()
    //         .map(|r| {
    //             self.global = self
    //                 .global
    //                 .iter()
    //                 .zip(&r.data)
    //                 .map(|(s, x)| s + x)
    //                 .collect::<Vec<_>>()
    //                 .to_vec()
    //         })
    //         .collect::<Vec<_>>()
    //         .to_vec();
    // }
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
