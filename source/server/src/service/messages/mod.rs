mod engine;

use self::engine::EngineService;
use crate::engine::channel::RequestSender;

#[derive(Debug)]
pub struct MessageHandler {
    engine: EngineService,
}

impl MessageHandler {
    pub fn new(tx: RequestSender) -> Self {
        MessageHandler {
            engine: EngineService::new(tx),
        }
    }
}
