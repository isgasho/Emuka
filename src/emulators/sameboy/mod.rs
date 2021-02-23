use std::time::Instant;

use crate::game;

use super::EmulatorCommand;

#[allow(warnings)]
mod bindings;
mod wrapper;
mod environment;
mod input;
mod audio;
mod video;


fn run_x_frames(x: u32) {
    let start = Instant::now();

    for _ in 0..x {
        wrapper::run_frame();
    }

    let elapsed = start.elapsed();

    println!("Duration of {} frames: {:?}ms", x, elapsed.as_millis());
}

pub static PATH: &str = "./game/lsdj4.5.1_TBC.gb";

#[derive(Debug, Default)]
pub struct SameBoyEmulator {
    game_path: Option<String>,
    save_path: Option<String>,
    instant: Option<std::time::Instant>,
    frames: usize
}


impl super::Emulator for SameBoyEmulator {
    fn init(&mut self) {
        self.instant = Some(Instant::now());
        wrapper::set_audio_frequency(crate::audio::SAMPLE_RATE);
        wrapper::set_environment_cb(environment::environment_callback);
        wrapper::set_input_poll_cb(input::input_poll);
        wrapper::set_input_state_cb(input::input_state);
        wrapper::set_audio_sample_cb(audio::audio_sample);
        wrapper::set_video_refresh_cb(video::video_refresh);
        wrapper::init();
    }

    fn load_game(&mut self, game_path: String) {
        self.save_path = Some(format!("{}.sav", &game_path));
        self.game_path = Some(game_path);
        
        let game_info = wrapper::GameInfo {
            path: self.game_path.as_ref().unwrap().clone()
        };

        wrapper::load_game(&game_info);
        wrapper::load_save(&self.save_path.as_ref().unwrap());
    }

    fn handle_command(&mut self, command: super::EmulatorCommand) -> bool {
        use EmulatorCommand::*;
        
        match command {
            RunFrame => {
                wrapper::run_frame();
                self.frames = self.frames + 1;
                if self.frames % 60 == 0 {
                    let elapsed = self.instant.unwrap().elapsed().as_millis();
                    println!("Ran 60 frames in {}ms", elapsed);
                    self.instant = Some(Instant::now());
                }
            },
            Input(input) => {
               let sb_input = wrapper::SameboyJoypadInput::from(input);
               input::store_input(sb_input);
            }
            Stop => return false
        };

        true
    }

    fn uninit(&mut self) {
        wrapper::save(self.save_path.as_ref().unwrap());
        wrapper::unload_game();
        wrapper::deinit();
    }
}

