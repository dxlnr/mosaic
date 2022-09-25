use async_trait::async_trait;

use crate::state_engine::{
    states::{IntoNextState, State, StateCondition},
    TransitionState,
};

#[derive(Debug)]
pub struct Idle;

#[async_trait]
impl StateCondition<Idle> for State<Idle> {
    async fn proceed(mut self) -> TransitionState {
        TransitionState::Pending(self.into())
    }
}

impl IntoNextState<Idle> for State<Idle> {
    fn into_next_state(mut self) -> State<Idle> {
        self.smpc.notify_idle();
        State::<Idle>::new(self.shared, self.smpc, Idle).into()
    }
}
