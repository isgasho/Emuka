pub mod sameboy;

use std::sync::Arc;

use crate::game::{Game, Save};

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

#[derive(Debug, Copy, Clone)]
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