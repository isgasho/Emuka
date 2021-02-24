use std::time::Instant;

use warp::ext::get;

use crate::game::{self, Game};

use super::EmulatorCommand;

#[allow(warnings)]
mod bindings;
mod wrapper;
mod environment;
mod input;
mod audio;
mod video;

pub static PATH: &str = "./game/lsdj4.5.1_TBC.gb";

#[derive(Debug, Default)]
pub struct SameBoyEmulator {
    game_path: Option<String>,
    save_path: Option<String>,
    instant: Option<std::time::Instant>,
    frames: usize,
    running: bool
}

impl SameBoyEmulator {
    fn load_game(&mut self, game: Box<dyn Game>) {
        self.game_path = Some(game.path().unwrap()
            .to_str().unwrap()
            .to_owned());
        
        let game_info = wrapper::GameInfo {
            path: self.game_path.as_ref().unwrap().clone()
        };

        wrapper::load_game(&game_info);
        println!("Game loaded");
    }

    fn load_save(&mut self, save: Box<dyn game::Save>) {
        self.save_path = Some(save.path().unwrap()
            .to_str().unwrap()
            .to_owned());

        wrapper::load_save(self.save_path.as_ref().unwrap());
        println!("Save loaded");
    }
}


impl super::Emulator for SameBoyEmulator {
    fn init(&mut self) {
        self.instant = Some(Instant::now());
        wrapper::set_audio_frequency(crate::audio::SAMPLE_RATE);
        wrapper::set_environment_cb(environment::environment_callback);
        wrapper::set_input_poll_cb(input::input_poll);
        wrapper::set_input_state_cb(input::input_state);
        wrapper::set_audio_sample_cb(audio::audio_sample);
        wrapper::set_video_refresh_cb();
        wrapper::init();
    }


    fn handle_command(&mut self, command: super::EmulatorCommand) -> bool {
        use EmulatorCommand::*;
        
        match command {
            RunFrame => {
                if self.running {
                    wrapper::run_frame();
                    if self.frames % 5 == 0 {
                        // wrapper::get_screen_data();
                    }
                }
                self.frames = self.frames + 1;
                // if self.frames % 60 == 0 {
                //     let elapsed = self.instant.unwrap().elapsed().as_millis();
                //     println!("Ran 60 frames in {}ms", elapsed);
                //     self.instant = Some(Instant::now());
                // }
            },
            Input(input) => {
               let sb_input = wrapper::SameboyJoypadInput::from(input);
               input::store_input(sb_input);
            }
            Stop => return false,
            LoadGame(game) => self.load_game(game),
            LoadSave(save) => self.load_save(save),
            Pause => self.running = false,
            Resume => self.running = true,
        };

        true
    }

    fn uninit(&mut self) {
        wrapper::save(self.save_path.as_ref().unwrap());
        wrapper::unload_game();
        wrapper::deinit();
    }

}

