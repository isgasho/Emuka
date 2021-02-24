use std::sync::atomic::{AtomicI16, Ordering};

use super::wrapper::*;

pub fn input_poll() {}

static INPUT_STATE: AtomicI16 = AtomicI16::new(0);

pub fn input_state() -> i16 {
    let value = INPUT_STATE.load(Ordering::Acquire);
    return value;
}

pub fn store_input(input: SameboyJoypadInput, pressed: bool) {
    let input_value = 1 << input as u32;
    if pressed {
        INPUT_STATE.fetch_or(input_value as i16, Ordering::Relaxed);
    } else {
        INPUT_STATE.fetch_and(!input_value as i16, Ordering::Relaxed);
    }
}
