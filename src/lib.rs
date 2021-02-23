#![macro_use]

use std::{path::PathBuf, sync::Arc};

use emulators::{Emulator, EmulatorCommand};
use game::{GameFromFile, SaveFile};
use tokio::time;
extern crate lazy_static;

pub mod emulators;
pub mod game;
pub mod audio;

pub async fn init() {
    let _stream = audio::init_audio();
    let mut emulator = emulators::sameboy::SameBoyEmulator::default();
    emulator.init();

    let game = Box::new(
        GameFromFile::new("LSDj", emulators::sameboy::PATH).unwrap()
    );

    let mut save_path = PathBuf::new();
    save_path.push(emulators::sameboy::PATH);
    save_path.set_extension("sav");

    let save = Box::new(
        SaveFile::new("LSDj", save_path.into_boxed_path()).unwrap()
    );



    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<emulators::EmulatorCommand>();
    let sender_interval = sender.clone();
    let sender_command = sender.clone();

    sender.send(EmulatorCommand::LoadGame(game)).unwrap();
    sender.send(EmulatorCommand::LoadSave(save)).unwrap();

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

        // time::sleep(time::Duration::from_secs(5)).await;
        // sender_command.send(EmulatorCommand::Pause).unwrap();

        // time::sleep(time::Duration::from_secs(5)).await;
        // sender_command.send(EmulatorCommand::Resume).unwrap();
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

    emulator.uninit();
}

