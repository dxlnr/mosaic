use async_trait::async_trait;

use crate::state_engine::{
    states::{IntoNextState, State, StateCondition, Idle},
    TransitionState,
};

#[derive(Debug)]
pub struct NewTask;

#[async_trait]
impl StateCondition<NewTask> for State<NewTask> {
    async fn proceed(mut self) -> TransitionState {
        TransitionState::Pending(self.into())
    }
}

impl IntoNextState<NewTask> for State<NewTask> {
    fn into_next_state(self) -> State<NewTask> {
        // self.smpc.notify_idle();
        State::<NewTask>::new(self.shared, self.smpc, NewTask).into()
    }
}

impl From<State<NewTask>> for State<Idle> {
    fn from(new_task: State<NewTask>) -> Self {
        State::new(new_task.shared, new_task.smpc, Idle).into_next_state()
    }
}