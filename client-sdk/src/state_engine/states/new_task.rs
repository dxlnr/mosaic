use async_trait::async_trait;
use tracing::debug;

use crate::state_engine::{
    states::{IntoState, State, StateCondition, Idle, Update},
    TransitionState,
};

#[derive(Debug)]
pub struct NewTask;

#[async_trait]
impl StateCondition<NewTask> for State<NewTask> {
    async fn proceed(mut self) -> TransitionState {
        debug!("Client runs in State: `NewTask`.");

        debug!("Client is selected for `UpdateTask`.");
        TransitionState::Complete(self.into_update().into())
    }
}

impl IntoState<NewTask> for State<NewTask> {
    fn into_state(mut self) -> State<NewTask> {
        self.smpc.notify_new_task();
        State::<NewTask>::new(self.shared, self.smpc, NewTask).into()
    }
}

impl From<State<NewTask>> for State<Idle> {
    fn from(new_task: State<NewTask>) -> Self {
        State::new(new_task.shared, new_task.smpc, Idle).into_state()
    }
}

impl State<NewTask> {
    fn into_update(self) -> State<Update> {
        State::<Update>::new(self.shared, self.smpc, Update).into()
    }
}