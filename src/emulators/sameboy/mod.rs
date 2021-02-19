#[allow(warnings)]
mod bindings;
mod wrapper;
mod environment;

pub fn init() {
    wrapper::set_environment_cb(environment::environment_callback);

    unsafe {
        bindings::retro_init();
        bindings::retro_load_game(std::ptr::null());
    }
}

