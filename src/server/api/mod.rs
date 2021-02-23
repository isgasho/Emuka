use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::UnboundedSender;
use warp::{Filter, Reply, filters::BoxedFilter};

use crate::emulators::EmulatorCommand;

mod v1;

#[derive(Clone)]
pub struct CommandSender {
    sender: Arc<Mutex<UnboundedSender<EmulatorCommand>>>
}

impl CommandSender {
    pub fn new(sender: UnboundedSender<EmulatorCommand>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender))
        }
    }

    pub fn send_command(&self, command: EmulatorCommand) {
        self.sender.lock().unwrap().send(command).unwrap();
    }
}

pub fn routes(sender: UnboundedSender<EmulatorCommand>) -> BoxedFilter<(impl Reply,)> {
    let sender = CommandSender::new(sender);

    v1::routes(sender.clone()).or(
        warp::path("v1")
        .and(v1::routes(sender))
    ).boxed()
}