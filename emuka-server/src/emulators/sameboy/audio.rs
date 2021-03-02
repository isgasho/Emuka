use crate::audio::StereoSample;
use crate::audio::SAMPLES_MAP;

pub fn audio_sample(left: i16, right: i16) {
    let mut lock = SAMPLES_MAP.lock().unwrap();
    let map = &mut *lock;

    for (_, queue) in map.iter_mut() {
        queue.push_back (StereoSample {left, right});
    }
}

