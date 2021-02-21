use std::sync::atomic::{AtomicI16, Ordering};

use super::wrapper::*;

pub fn input_poll() {
    // println!("Input polled");
}

static INPUT_STATE: AtomicI16 = AtomicI16::new(0);

pub fn input_state() -> i16 {
    let value = INPUT_STATE.load(Ordering::Acquire);
    if value != 0 {
        INPUT_STATE.store(0, Ordering::Release);
    }

    return value;
}

pub fn store_input(input: SameboyJoypadInput) {
    let input_value = 1 << input as u32;
    INPUT_STATE.fetch_or(input_value as i16, Ordering::Relaxed);
}
