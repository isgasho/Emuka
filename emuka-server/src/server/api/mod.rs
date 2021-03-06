use std::sync::{Arc, Mutex, mpsc::Sender};

use tokio::sync::mpsc::UnboundedSender;
use warp::{Filter, Reply, filters::BoxedFilter};

use crate::{audio::AudioCommand, emulators::EmulatorCommand};

pub mod v1;

#[derive(Clone)]
pub struct EmulatorCommandSender {
    sender: Arc<Mutex<UnboundedSender<EmulatorCommand>>>
}

impl EmulatorCommandSender {
    pub fn new(sender: UnboundedSender<EmulatorCommand>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender))
        }
    }

    pub fn send_command(&self, command: EmulatorCommand) {
        self.sender.lock().unwrap().send(command).unwrap();
    }
}

#[derive(Clone)]
pub struct AudioCommandSender {
    sender: Arc<Mutex<Sender<AudioCommand>>>
}

impl AudioCommandSender {
    pub fn new(sender: Sender<AudioCommand>) -> Self {
        Self {
            sender: Arc::new(Mutex::new(sender))
        }
    }

    pub fn send_command(&self, command: AudioCommand) {
        self.sender.lock().unwrap().send(command).unwrap();
    }
}

pub fn routes(emulator_sender: UnboundedSender<EmulatorCommand>, audio_sender: Sender<AudioCommand>) -> BoxedFilter<(impl Reply,)> {
    let emulator_sender = EmulatorCommandSender::new(emulator_sender);
    let audio_sender = AudioCommandSender::new(audio_sender);

    v1::routes(emulator_sender.clone(), audio_sender.clone()).or(
        warp::path("v1")
        .and(v1::routes(emulator_sender, audio_sender))
    ).boxed()
}