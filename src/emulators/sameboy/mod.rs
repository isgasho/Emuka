use std::time::Instant;

#[allow(warnings)]
mod bindings;
mod wrapper;
mod environment;
mod input;
mod audio;
mod video;


fn run_60_frames() {
    let start = Instant::now();

    for _ in 0..60 {
        wrapper::run_frame();
    }

    let elapsed = start.elapsed();

    println!("Duration of 60 frames: {:?}ms", elapsed.as_millis());
}

pub fn init() {
    wrapper::set_environment_cb(environment::environment_callback);
    wrapper::set_input_poll_cb(input::input_poll);
    wrapper::set_input_state_cb(input::input_state);
    wrapper::set_audio_sample_cb(audio::audio_sample);
    wrapper::set_video_refresh_cb(video::video_refresh);

    wrapper::init();

    let game_info = wrapper::GameInfo {
        path: String::from("./game/lsdj.gb")
    };

    wrapper::load_game(&game_info);
    
    run_60_frames();
}

