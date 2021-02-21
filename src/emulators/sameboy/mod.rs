use std::time::Instant;

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

pub fn init() {
    let stream = audio::init_audio();
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
    
    run_x_frames(1200);
}

