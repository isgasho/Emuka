#![macro_use]

use emulators::{Emulator, EmulatorCommand};
use tokio::time;
extern crate lazy_static;

pub mod emulators;
pub mod game;
pub mod audio;

pub async fn init() {
    let _stream = audio::init_audio();
    let mut emulator = emulators::sameboy::SameBoyEmulator::default();
    emulator.init();
    emulator.load_game(emulators::sameboy::PATH.to_owned());

    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<emulators::EmulatorCommand>();
    let sender_interval = sender.clone();
    let sender_command = sender.clone();

    let mut interval = time::interval(time::Duration::from_nanos(1_000_000_000 / 60));


    tokio::spawn(async move {
        loop {
            interval.tick().await;
            sender_interval.send(EmulatorCommand::RunFrame).unwrap();
        }
    });

    tokio::spawn(async move {
        time::sleep(time::Duration::from_secs(5)).await;
        sender_command.send(EmulatorCommand::Input(emulators::EmulatorJoypadInput::START)).unwrap();
    });

    
    loop {
        let command = receiver.recv().await;
        
        match command {
            Some(command) => if !emulator.handle_command(command) {
                break;
            },
            None => break
        };
    }
}

