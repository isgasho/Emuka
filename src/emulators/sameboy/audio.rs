use std::{collections::VecDeque, sync::{Mutex, RwLock, atomic::{AtomicUsize, Ordering}}};

use cpal::{OutputCallbackInfo, SampleFormat, SampleRate, Stream, SupportedStreamConfigRange, traits::{DeviceTrait, HostTrait, StreamTrait}};
use lazy_static::lazy_static;
use crate::audio::{SAMPLES, StereoSample};


pub fn audio_sample(left: i16, right: i16) {
    let mut lock = SAMPLES.lock().unwrap();
    (*lock).push_back (StereoSample {left, right});
}
