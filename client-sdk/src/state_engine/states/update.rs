use async_trait::async_trait;
// use tracing::warn;

use crate::state_engine::{
    states::{IntoNextState, NewTask, State, StateCondition},
    TransitionState,
};

#[derive(Debug)]
pub struct Update;

#[async_trait]
impl StateCondition<Update> for State<Update> {
    async fn proceed(mut self) -> TransitionState {
        TransitionState::Pending(self.into())
    }
}

impl IntoNextState<Update> for State<Update> {
    fn into_next_state(self) -> State<Update> {
        // self.smpc.notify_idle();
        State::<Update>::new(self.shared, self.smpc, Update).into()
    }
}

impl From<State<Update>> for State<NewTask> {
    fn from(new_task: State<Update>) -> Self {
        State::new(new_task.shared, new_task.smpc, NewTask).into_next_state()
    }
}
