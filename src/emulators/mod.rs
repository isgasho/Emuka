pub mod sameboy;

use crate::game::Game;

pub trait Emulator {
    fn init(&mut self);
    fn load_game(&mut self, game: dyn Game);
}

