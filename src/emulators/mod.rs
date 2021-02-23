pub mod sameboy;

use crate::game::Game;

pub trait Emulator {
    fn init(&mut self);
    fn load_game(&mut self, game_path: String);
    fn handle_command(&mut self, command: EmulatorCommand) -> bool;
    fn uninit(&mut self);
}

#[derive(Debug, Copy, Clone)]
pub enum EmulatorCommand {
    RunFrame,
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