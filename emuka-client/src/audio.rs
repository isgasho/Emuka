use core::panic;
use std::{collections::VecDeque, sync::Mutex};

use avro_rs::{Reader, types::Value};
use cpal::{OutputCallbackInfo, SampleFormat, SampleRate, Stream, SupportedStreamConfigRange, traits::{DeviceTrait, HostTrait, StreamTrait}};
use eyre::Result;

use emuka_server::{audio::{SAMPLE_RATE, StereoSample, VecStereoWrapper}, server::api::v1::api::{AUDIO_DATA_API_SCHEMA, AudioRegisterApi}};
use lazy_static::lazy_static;
use tokio::time;

lazy_static! {
    pub(crate) static ref SAMPLES: Mutex<VecDeque<StereoSample>> = Mutex::new(VecDeque::with_capacity((SAMPLE_RATE * 20) as usize));
}

pub fn init_audio_stream() -> Stream {
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
        cpal::SampleFormat::F32 => run::<f32>( &device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>( &device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>( &device, &config.into()),
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



fn decode_avro_data_samples(avro_data: &[u8]) -> Vec<u8> {
    let reader = Reader::with_schema(&AUDIO_DATA_API_SCHEMA, &avro_data[..]).unwrap();
    for record in reader {
        match record.unwrap() {
            Value::Record(fields) => {
                let bytes = &fields.get(0).unwrap().1;
                match bytes {
                    Value::Bytes(data) => {
                        return data.clone()
                    },
                    _ => panic!()
                }
            },
            _  => panic!()
        }
    }
    return Vec::new();
}

fn write_encoded_samples(avro_data: &[u8]) {
    let decoded = decode_avro_data_samples(avro_data);
    let wrapper = VecStereoWrapper::from(decoded);

    println!("{:?}", wrapper.inner.as_ref().unwrap().len());

    if let Some(data) = wrapper.inner {
        let mut lock = SAMPLES.lock().unwrap();
        lock.extend(data.into_iter());
    }
}

pub async fn init_audio_requests(base: String) -> Result<()> {
    let audio_id = reqwest::get(&format!("{}/api/v1/audio/register", base))
        .await?
        .json::<AudioRegisterApi>()
        .await?
        .id;
    
    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_millis(10));
        
        loop {
            interval.tick().await;

            let response =
            reqwest::get(&format!("{}/api/v1/audio/get/{}", base, audio_id))
            .await;

            match response {
                Err(err) => {
                    println!("{}", err);
                    continue;
                },
                Ok(response) => {
                    let data = response.bytes().await;
                    match data {
                        Err(err) => {
                            println!("{}", err);
                            continue;
                        },
                        Ok(data) => {
                            write_encoded_samples(&data);
                        }
                    }
                }
            };
            
        }
    });

    Ok(())
}
