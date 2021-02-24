pub mod sameboy;

use std::{path::PathBuf};

use tokio::{sync::mpsc::{UnboundedSender, unbounded_channel}, time};

use crate::game::{Game, GameFromFile, Save, SaveFile};

use self::sameboy::SameBoyEmulator;

pub trait Emulator {
    fn init(&mut self);
    fn handle_command(&mut self, command: EmulatorCommand) -> bool;
    fn uninit(&mut self);
}

#[derive(Debug)]
pub enum EmulatorCommand {
    LoadGame(Box<dyn Game>),
    LoadSave(Box<dyn Save>),
    RunFrame,
    Pause,
    Resume,
    Input(EmulatorJoypadInput),
    Stop,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub enum EmulatorJoypadInput {
    A,
    B,
    SELECT,
    START,
    UP,
    DOWN,
    RIGHT,
    LEFT
}

pub async fn init() -> UnboundedSender<EmulatorCommand> {
    let mut emulator = SameBoyEmulator::default();
    emulator.init();

    let (sender, mut receiver) = unbounded_channel::<EmulatorCommand>();
    let sender_interval = sender.clone();

    let mut interval = time::interval(time::Duration::from_nanos(1_000_000_000 / 60));


    tokio::spawn(async move {
        loop {
            interval.tick().await;
            sender_interval.send(EmulatorCommand::RunFrame).unwrap();
        }
    });
    
    tokio::spawn( async move {
        loop {
            let command = receiver.recv().await;
            
            match command {
                Some(command) => if !emulator.handle_command(command) {
                    break;
                },
                None => break
            };
        }

        println!("Emulator uninit");
        emulator.uninit();
    });

    sender
}