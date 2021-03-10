use std::{collections::HashMap, time::Instant};
use eyre::Result;
use tokio::sync::oneshot::Sender;
use lazy_static::lazy_static;
use onig::Regex;

use crate::game::{self, Game};

use super::{EmulatorCommand, EmulatorInternalCommand, EmulatorInternalCommandResult, EmulatorInternalCommandResults};

#[allow(warnings)]
mod bindings;
mod wrapper;
mod environment;
mod input;
mod audio;
mod video;

lazy_static! {
    static ref RE_IS_ASSIGNMENT: Regex = Regex::new(r"(?<![>!<=])=(?!=)").unwrap();
}
#[derive(Debug)]
pub struct SameBoyEmulator {
    game_path: Option<String>,
    save_path: Option<String>,
    running: bool,

    before: Option<std::time::Instant>,
    frames: usize,
    frame_interval: i128,
    delta: i128,
    skip_next: bool
}

impl Default for SameBoyEmulator {
    fn default() -> Self {
        Self {
            game_path: None,
            save_path: None,
            running: false,
            before: None,
            frames: 0,
            frame_interval: 0,
            delta: 0,
            skip_next: false
        }
    }
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

    fn unload_game(&mut self) {
        self.running = false;
        self.game_path = None;
        wrapper::unload_game();
    }

    fn load_save(&mut self, save: Box<dyn game::Save>) {
        if self.game_path.is_none() {
            return;
        }

        if save.can_write() {
            self.save_path = Some(save.path().unwrap()
                .to_str().unwrap()
                .to_owned());
            
            wrapper::load_save(self.save_path.as_ref().unwrap());
            println!("Save loaded");
        } else {
            println!("Read only save, unimplemented for now")
        }
    }

    fn run_frame(&mut self) {
        if self.frame_interval == 0 {
            return;
        }

        if self.running && !self.skip_next {
            wrapper::run_frame();
        }

        self.skip_next = false;

        self.frames = self.frames + 1;
        
        match self.before {
            Some(before) => {
                let now = Instant::now();
                let elapsed = before.elapsed().as_nanos() as i128 ;
                self.delta = self.delta + (elapsed - self.frame_interval);
                self.before = Some(now);

                if self.frames % 60 == 0 {
                    println!("FrameInter : {:>10}", self.frame_interval);
                    println!("Difference : {:>10}", self.delta);
                    println!("FrameInter!: {:>10}", -(self.frame_interval));
                    println!("elapsed    : {:>10}", (elapsed - self.frame_interval));
                    println!();
                    
                    if self.delta > self.frame_interval {
                        println!("=========== Too slow! ===========");
                        self.run_frame();
                    }
    
                    if self.delta < -(self.frame_interval) {
                        println!("=========== Too fast! ===========");
                        self.skip_next = true;
                        self.delta = self.delta + self.frame_interval;
                    }
                }

            }
            None => {
                self.before = Some(Instant::now())
            }
        }
    }

    fn save(&self) {
        if let Some(save_path) = self.save_path.as_ref() {
            wrapper::save(&save_path);
        }
    }

    fn run_stealth(&mut self, jump_location: u32, mut state: HashMap<String, u32>) -> Result<HashMap<String, u32>> {
        if !self.running {
            return Err(eyre::Report::msg("Game is not running!"));
        }

        let converted = jump_location as u16;
        wrapper::run_stealth(converted, &mut state).map(|_| state)
    }

    fn run_stealth_and_send(&mut self, jump_location: u32, state: HashMap<String, u32>, sender: Sender<Option<HashMap<String, u32>>>) {
        let result = self.run_stealth(jump_location, state);

        match result {
            Ok(state) => sender.send(Some(state)).unwrap(),
            Err(_) => sender.send(None).unwrap()
        }
    }

    fn read_memory(&mut self, request: String) -> Option<String> {
        if RE_IS_ASSIGNMENT.find(&request).is_some() {
            return None;
        }

        wrapper::evaluate(request)
    }

    fn read_memory_and_send(&mut self, request: String, sender: Sender<Option<String>>) {
        sender.send(self.read_memory(request)).unwrap()
    }

    fn write_memory(&mut self, request: String) -> Option<String> {
        if RE_IS_ASSIGNMENT.find(&request).is_none() {
            return None;
        }

        wrapper::evaluate(request)
    }

    fn write_memory_and_send(&mut self, request: String, sender: Sender<Option<String>>) {
        sender.send(self.write_memory(request)).unwrap()
    }

    fn burst(&mut self, commands: Vec<EmulatorInternalCommand>, sender: Sender<EmulatorInternalCommandResults>) {
        let mut results: EmulatorInternalCommandResults = Vec::with_capacity(commands.len());
        let commands_len = commands.len();

        for command in commands {
            match command {
                EmulatorInternalCommand::ReadMemory{ request } => {
                    let value = self.read_memory(request);
                    results.push(value.map(|string| EmulatorInternalCommandResult::ReadMemory(string)));
                }
                EmulatorInternalCommand::WriteMemory{ request } => {
                    let value = self.write_memory(request);
                    results.push(value.map(|string| EmulatorInternalCommandResult::WriteMemory(string)));
                }
                EmulatorInternalCommand::RunStealth{ jump_location, state } => {
                    match self.run_stealth(jump_location, state) {
                        Ok(state) => results.push(Some(EmulatorInternalCommandResult::RunStealth(state))),
                        Err(_) => results.push(None)
                    };
                }
                EmulatorInternalCommand::RunFrame => {
                    self.run_frame();
                    results.push(None)
                }
            }
        }

        assert!(commands_len == results.len());

        sender.send(results).unwrap();
    }
}

const FRAME_RATE: f32 = 59.7154;

impl super::Emulator for SameBoyEmulator {
    fn init(&mut self) {
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
            RunFrame => self.run_frame(),
            RunStealth(jump_location, state, sender) => self.run_stealth_and_send(jump_location, state, sender),
            ReadMemory(request, sender) => self.read_memory_and_send(request, sender),
            WriteMemory(request, sender) => self.write_memory_and_send(request, sender),
            GetScreenData(sender) => sender.send(wrapper::get_screen_data()).unwrap(),
            Input((input, pressed)) => {
               let sb_input = wrapper::SameboyJoypadInput::from(input);
               input::store_input(sb_input, pressed);
            }
            Save => self.save(),
            Stop => return false,
            LoadGame(game) => self.load_game(game),
            UnloadGame => self.unload_game(),
            LoadSave(save) => self.load_save(save),
            Pause => self.running = false,
            Resume => self.running = true,
            Burst(commands, sender) => self.burst(commands, sender)
        };

        true
    }

    fn uninit(&mut self) {
        wrapper::save(self.save_path.as_ref().unwrap());
        wrapper::unload_game();
        wrapper::deinit();
    }

    fn frame_rate(&self) -> f32 {
        FRAME_RATE
    }

    fn set_frame_interval(&mut self, frame_interval: i128) {
        self.frame_interval = frame_interval;
    }
}

