use core::panic;
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::sync::mpsc::*;


use uuid::Uuid;

use lazy_static::lazy_static;

use cpal::{OutputCallbackInfo, SampleFormat, SampleRate, Stream, SupportedStreamConfigRange, traits::{DeviceTrait, HostTrait, StreamTrait}};

#[derive(Debug, Copy, Clone, Deserialize, Serialize)]
pub struct StereoSample {
    pub left: i16,
    pub right: i16
}

impl StereoSample {
    pub fn to_byte_array(&self) -> [u8; 4] {
        let left = self.left.to_le_bytes();
        let right = self.right.to_le_bytes();

        let mut a = [0u8; 4];

        &a[..2].copy_from_slice(&left);
        &a[2..].copy_from_slice(&right);

        a
    }

    pub fn from_byte_array(data: &[u8]) -> Self {
        assert!(data.len() == 4);

        let mut left_data = [0u8; 2];
        left_data.copy_from_slice(&data[0..2]);
        let left: i16 = i16::from_le_bytes(left_data);

        let mut right_data = [0u8; 2];
        right_data.copy_from_slice(&data[2..4]);
        let right: i16 = i16::from_le_bytes(left_data);

        return Self {
            left, right
        }
    }
}




pub struct VecStereoWrapper {
    pub inner: Option<Vec<StereoSample>>
}

impl From<Vec<u8>> for VecStereoWrapper {
    fn from(bytes: Vec<u8>) -> Self {
        let mut samples: Vec<StereoSample> = vec![];
        for chunk in bytes.chunks_exact(4) {
            samples.push(StereoSample::from_byte_array(chunk));
        }

        Self {
            inner: Some(samples) 
        }
    }
}

impl Into<Vec<u8>> for VecStereoWrapper {
    fn into(self) -> Vec<u8> {
        match self.inner {
            None => Vec::new(),
            Some(samples) => {
                let mut samples_data: Vec<u8> = vec![];
                
                for sample in samples {
                    samples_data.extend_from_slice(&sample.to_byte_array());
                }

                samples_data
            }
        }
    }
} 

pub static SAMPLE_RATE: u32 = 48000;

lazy_static! {
    pub(crate) static ref SAMPLES_MAP: Mutex<HashMap<Uuid, VecDeque<StereoSample>>> = Mutex::new(HashMap::new());
}

pub enum AudioCommand {
    Resume,
    Pause,
}

pub fn init() -> Sender<AudioCommand> {
    let (sender, receiver) = channel::<AudioCommand>();

    tokio::spawn(async move {
        let stream = init_audio_stream();
        loop {
            let command = receiver.recv();
            
            match command {
                Ok(command) => match command {
                    AudioCommand::Resume => {
                        stream.play().unwrap();
                    },
                    AudioCommand::Pause => {
                        stream.pause().unwrap();
                    },
                },
                Err(err) => {
                    println!("{:?}", err);
                    break;
                }
            };
        }
        println!("Stream dropped ?");
    });


    sender
}


fn init_audio_stream() -> Stream {
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

    let id = Uuid::new_v4();
    {
        let mut lock = SAMPLES_MAP.lock().unwrap();
        lock.insert(id, VecDeque::with_capacity((SAMPLE_RATE * 20) as usize));
    }

    let stream = match sample_format {
        cpal::SampleFormat::F32 => run::<f32>( &device, &config.into(), id),
        cpal::SampleFormat::I16 => run::<i16>( &device, &config.into(), id),
        cpal::SampleFormat::U16 => run::<u16>( &device, &config.into(), id),
    };

    stream.play().unwrap();

    return stream;
}

fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, id: Uuid) -> Stream where
T: cpal::Sample {
    let channels = config.channels as usize;


    device.build_output_stream(
        &config, 
        move |data: &mut [T], info: &OutputCallbackInfo| {
            write_data(data, info, channels, id);
        },
        |err| println!("{:?}", err)
    ).unwrap()
}


fn write_data<T>(data: &mut [T], _: &OutputCallbackInfo, channels: usize, id: Uuid) where
T: cpal::Sample {
    assert!(channels == 2);

    let mut lock = SAMPLES_MAP.lock().unwrap();
    let samples = &mut *lock.get_mut(&id).unwrap();

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


