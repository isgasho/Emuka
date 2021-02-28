use crate::audio::{SAMPLES, StereoSample};

pub fn audio_sample(left: i16, right: i16) {
    let mut lock = SAMPLES.lock().unwrap();
    (*lock).push_back (StereoSample {left, right});
}
