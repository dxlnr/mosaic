use async_trait::async_trait;
use tracing::debug;

use crate::state_engine::{
    states::{Idle, IntoState, NewTask, State, StateCondition},
    TransitionState,
};

#[derive(Debug)]
pub struct Update;

#[async_trait]
impl StateCondition<Update> for State<Update> {
    async fn proceed(mut self) -> TransitionState {
        debug!("Client runs in State: `Update`.");
        TransitionState::Complete(self.into_idle().into())
    }
}

impl IntoState<Update> for State<Update> {
    fn into_state(self) -> State<Update> {
        // self.smpc.notify_idle();
        State::<Update>::new(self.shared, self.smpc, Update).into()
    }
}

impl From<State<Update>> for State<NewTask> {
    fn from(new_task: State<Update>) -> Self {
        State::new(new_task.shared, new_task.smpc, NewTask).into_state()
    }
}

impl State<Update> {
    fn into_idle(self) -> State<Idle> {
        State::new(self.shared, self.smpc, Idle).into_state()
    }
}