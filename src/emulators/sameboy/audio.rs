use std::{collections::VecDeque, sync::{Mutex, RwLock, atomic::{AtomicUsize, Ordering}}};

use cpal::{OutputCallbackInfo, SampleFormat, SampleRate, Stream, SupportedStreamConfigRange, traits::{DeviceTrait, HostTrait, StreamTrait}};
use lazy_static::lazy_static;


struct StereoSample {
    pub left: i16,
    pub right: i16
}

static SAMPLE_RATE: u32 = 384000;
lazy_static! {
    static ref SAMPLES: Mutex<VecDeque<StereoSample>> = Mutex::new(VecDeque::with_capacity(SAMPLE_RATE as usize * 20usize));
}


pub fn audio_sample(left: i16, right: i16) {
    let mut lock = SAMPLES.lock().unwrap();
    (*lock).push_back (StereoSample {left, right});
}


pub fn init_audio() -> Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    println!("{}", device.name().unwrap());
    let supported_configs_range = device.supported_output_configs()
    .expect("error while querying configs");
    let appropriate_configs: Vec<SupportedStreamConfigRange> = supported_configs_range
    .filter(|conf| conf.channels() == 2)
    .filter(|conf| conf.max_sample_rate() >= SampleRate(SAMPLE_RATE))
    .collect();

    println!("number of appropriate configs: {}", appropriate_configs.len());

    if appropriate_configs.is_empty() {
        panic!("Unsupported on this host device")
    }

    let perfect_config = appropriate_configs.iter()
        .find(|conf| conf.sample_format() == SampleFormat::I16);

    let supported_config = match perfect_config {
        Some(config) => {
            println!("Perfect config found!");
            config.clone()
        },
        None => appropriate_configs.get(0).unwrap().clone()
    }.with_sample_rate(SampleRate(SAMPLE_RATE));

    let config = supported_config;
    println!("{:?}", config);
    let sample_format = config.sample_format();

    let stream = match sample_format {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
    };

    stream.play().unwrap();

    return stream;
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Stream where
T: cpal::Sample {
    let channels = config.channels as usize;


    device.build_output_stream(
        &config, 
        move |data: &mut [T], info: &OutputCallbackInfo| {
            write_data(data, info, channels);
        },
        |err| println!("{:?}", err)
    ).unwrap()
}

fn write_data<T>(data: &mut [T], _: &OutputCallbackInfo, channels: usize) where
T: cpal::Sample {
    assert!(channels == 2);

    let mut lock = SAMPLES.lock().unwrap();
    let samples = &mut *lock;

    for frame in data.chunks_mut(channels) {
        let next = samples.pop_front();
        match next {
            Some(value) => {
                let left: T = cpal::Sample::from::<i16>(&value.left);
                let right: T = cpal::Sample::from::<i16>(&value.right);
                let mut frame_iter = frame.iter_mut();
                *frame_iter.next().unwrap() = left;
                *frame_iter.next().unwrap() = right;
            },
            None => break
        }
    }
}
