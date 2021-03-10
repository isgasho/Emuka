pub mod sameboy;

use std::collections::HashMap;

use tokio::{sync::mpsc::{UnboundedSender, unbounded_channel}, time};
use tokio::sync::oneshot::Sender;

use crate::game::{Game, Save};

use self::sameboy::SameBoyEmulator;

pub trait Emulator {
    fn init(&mut self);
    fn frame_rate(&self) -> f32;
    fn set_frame_interval(&mut self, frame_interval: i128);
    fn handle_command(&mut self, command: EmulatorCommand) -> bool;
    fn uninit(&mut self);
}

#[derive(Debug)]
pub enum EmulatorCommand {
    LoadGame(Box<dyn Game>),
    UnloadGame,
    LoadSave(Box<dyn Save>),
    RunFrame,
    RunStealth(u32, HashMap<String, u32>, Sender<Option<HashMap<String, u32>>>),
    ReadMemory(String, Sender<Option<String>>),
    Save,
    GetScreenData(Sender<Option<ScreenData>>),
    Pause,
    Resume,
    Input((EmulatorJoypadInput, bool)),
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
#[derive(Debug, Deserialize, Clone)]
pub struct ScreenData {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32
}


pub async fn init() -> UnboundedSender<EmulatorCommand> {
    let mut emulator = SameBoyEmulator::default();
    emulator.init();
    
    let (sender, mut receiver) = unbounded_channel::<EmulatorCommand>();
    let sender_interval = sender.clone();
    
    let frame_rate = emulator.frame_rate();
    let frame_interval: u64 = (1_000_000_000f32 / frame_rate) as u64;
    emulator.set_frame_interval(frame_interval as i128);
    
    let mut interval = time::interval(time::Duration::from_nanos(frame_interval));

    
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