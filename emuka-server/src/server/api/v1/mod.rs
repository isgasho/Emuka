pub mod api;
use crate::{audio::{AudioCommand, VecStereoWrapper}, emulators::ScreenData, server::api::v1::api::*};

use std::{collections::{HashMap, VecDeque}, convert::TryInto};

use color_eyre::Report;
use tokio::sync::oneshot;
use warp::{Filter, Reply, filters::BoxedFilter};
use uuid::Uuid;
use avro_rs::{Codec, Writer, types::Record};

use crate::{audio::{SAMPLE_RATE, StereoSample}, emulators::{EmulatorCommand, EmulatorJoypadInput}, game::{GameFromFile, SaveFile}};
use crate::audio::SAMPLES_MAP;


use super::{AudioCommandSender, EmulatorCommandSender};

async fn load_game(
    game_api: GameFromFileApi,
    emulator_sender: EmulatorCommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    let result: Result<GameFromFile, Report> = game_api.try_into();

    let reply = match result {
        Ok(game) => {
            emulator_sender.send_command(EmulatorCommand::LoadGame(Box::from(game)));
            Ok(warp::reply::with_status(warp::reply(), warp::http::StatusCode::OK))
        },
        Err(err) => {
            eprintln!("{}", err);
            Ok(warp::reply::with_status(warp::reply(), warp::http::StatusCode::BAD_REQUEST))
        }
    };
    reply
}

async fn unload_game(
    emulator_sender: EmulatorCommandSender,
    audio_sender: AudioCommandSender,
) -> Result<impl warp::Reply, warp::Rejection> {
    audio_sender.send_command(AudioCommand::Pause);
    emulator_sender.send_command(EmulatorCommand::UnloadGame);
    Ok(warp::reply())
}

async fn load_save(
    save_api: SaveFromFileApi,
    sender: EmulatorCommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    let result: Result<SaveFile, Report> = save_api.try_into();

    let reply = match result {
        Ok(save) => {
            sender.send_command(EmulatorCommand::LoadSave(Box::from(save)));
            Ok(warp::reply::with_status(warp::reply(), warp::http::StatusCode::OK))
        },
        Err(err) => {
            eprintln!("{}", err);
            Ok(warp::reply::with_status(warp::reply(), warp::http::StatusCode::BAD_REQUEST))
        }
    };
    reply
}

async fn resume(
    emulator_sender: EmulatorCommandSender,
    audio_sender: AudioCommandSender,
) -> Result<impl warp::Reply, warp::Rejection> {
    audio_sender.send_command(AudioCommand::Resume);
    emulator_sender.send_command(EmulatorCommand::Resume);
    Ok(warp::reply())
}

async fn input(
    input_api: EmulatorJoypadInputApi,
    sender: EmulatorCommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    let input: (EmulatorJoypadInput, bool) = input_api.into();
    sender.send_command(EmulatorCommand::Input(input));
    Ok(warp::reply())
}

async fn get_screen_data(
    sender: EmulatorCommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    let (os_sender, os_receiver) = oneshot::channel::<Option<ScreenData>>();
    sender.send_command(EmulatorCommand::GetScreenData(os_sender));
    let value = os_receiver.await.unwrap();
    let screen_data = ScreenDataApi::from(value);
    
    
    let mut writer = Writer::with_codec(&api::SCREEN_DATA_API_SCHEMA, Vec::new(), Codec::Snappy);
    let mut record = Record::new(writer.schema()).unwrap();

    record.put("screen", screen_data.screen);
    record.put("width", screen_data.width as i32);
    record.put("height", screen_data.height as i32);

    writer.append(record).unwrap();
    let data = writer.into_inner().unwrap();

    Ok(warp::http::Response::new(data))
}

async fn save(
    sender: EmulatorCommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    sender.send_command(EmulatorCommand::Save);
    Ok(warp::reply())
}

async fn register_audio_queue() -> Result<impl warp::Reply, warp::Rejection> {
    let id = Uuid::new_v4();
    
    {
        let mut lock = SAMPLES_MAP.lock().unwrap();
        let map = &mut *lock;
        map.insert(id, VecDeque::with_capacity((SAMPLE_RATE * 20) as usize));
    }

    let audio_register = AudioRegisterApi {id};

    Ok(warp::reply::json(&audio_register))
}

async fn get_audio_samples (
    request: Uuid
) -> Result<impl warp::Reply, warp::Rejection> {
    let data = {
        let mut lock = SAMPLES_MAP.lock().unwrap();
        let map = &mut *lock;
        let queue = map.get_mut(&request);
        queue.map(|q| q.drain(..).collect::<Vec<StereoSample>>())
    };

    let mut writer = Writer::with_codec(&api::AUDIO_DATA_API_SCHEMA, Vec::new(), Codec::Snappy);
    let mut record = Record::new(writer.schema()).unwrap();


    let bytes: Vec<u8> = match data {
        Some(samples) => {
            let wrapper = VecStereoWrapper {
                inner: Some(samples)
            };
            wrapper.into()
        }
        None => {
            Vec::new()
        }
    };
    
    record.put("data", bytes);

    writer.append(record).unwrap();
    let data = writer.into_inner().unwrap();

    Ok(warp::http::Response::new(data))
}

async fn run_stealth (
    run_stealth_api: RunStealthRequestApi,
    emulator_sender: EmulatorCommandSender
)  -> Result<impl warp::Reply, warp::Rejection> {
    let (os_sender, os_receiver) = oneshot::channel::<Option<HashMap<String, u32>>>();
    emulator_sender.send_command(EmulatorCommand::RunStealth(run_stealth_api.jump_location, run_stealth_api.state, os_sender));

    let value = os_receiver.await.unwrap();

    match value {
        Some(data) => {
            let response = RunStealthResponseApi::new(data);
            Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK).into_response())
        }
        None => {
            Ok(warp::reply::with_status(warp::reply(), warp::http::StatusCode::BAD_REQUEST).into_response())
        }
    }
}

async fn read_memory (
    request: ReadMemoryRequestApi,
    emulator_sender: EmulatorCommandSender
)  -> Result<warp::reply::Response, warp::Rejection> {
    let (os_sender, os_receiver) = oneshot::channel::<Option<String>>();

    emulator_sender.send_command(EmulatorCommand::ReadMemory(request.request, os_sender));

    let value = os_receiver.await.unwrap();

    match value {
        Some(data) => {
            let response = ReadMemoryResponseApi::new(data);
            Ok(warp::reply::with_status(warp::reply::json(&response), warp::http::StatusCode::OK).into_response())
        }
        None => {
            Ok(warp::reply::with_status(warp::reply(), warp::http::StatusCode::BAD_REQUEST).into_response())
        }
    }
}

pub fn routes(emulator_sender: EmulatorCommandSender, audio_sender: AudioCommandSender) -> BoxedFilter<(impl Reply,)> {
    let emulator_command_filter = warp::any().map(move || emulator_sender.clone());
    let audio_command_filter = warp::any().map(move || audio_sender.clone());
    
    let load_game_f = warp::post()
        .and(warp::path("game"))
        .and(warp::path("load"))
        .and(warp::path::end())
        .and(post_json::<GameFromFileApi>())
        .and(emulator_command_filter.clone())
        .and_then(load_game);
    
    let unload_game_f = warp::get()
        .and(warp::path("game"))
        .and(warp::path("unload"))
        .and(warp::path::end())
        .and(emulator_command_filter.clone())
        .and(audio_command_filter.clone())
        .and_then(unload_game);
    

    let load_save_f = warp::post()
        .and(warp::path("save"))
        .and(warp::path("load"))
        .and(warp::path::end())
        .and(post_json::<SaveFromFileApi>())
        .and(emulator_command_filter.clone())
        .and_then(load_save);

    let resume_f = warp::get()
        .and(warp::path("resume"))
        .and(warp::path::end())
        .and(emulator_command_filter.clone())
        .and(audio_command_filter.clone())
        .and_then(resume);

    let input_f = warp::post()
        .and(warp::path("input"))
        .and(warp::path::end())
        .and(post_json::<EmulatorJoypadInputApi>())
        .and(emulator_command_filter.clone())
        .and_then(input);

    let get_screen_data_f = warp::get()
        .and(warp::path("screen"))
        .and(warp::path::end())
        .and(emulator_command_filter.clone())
        .and_then(get_screen_data);

    let safe_f = warp::get()
        .and(warp::path("save"))
        .and(warp::path("save"))
        .and(warp::path::end())
        .and(emulator_command_filter.clone())
        .and_then(save);

    let register_audio_queue_f = warp::get()
        .and(warp::path("audio"))
        .and(warp::path("register"))
        .and(warp::path::end())
        .and_then(register_audio_queue);

    let get_audio_samples_f = warp::get()
        .and(warp::path("audio"))
        .and(warp::path("get"))
        .and(warp::path::param().map(|id: Uuid| id))
        .and(warp::path::end())
        .and_then(get_audio_samples);

    let run_stealth_f = warp::post()
        .and(warp::path("internal"))
        .and(warp::path("run_stealth"))
        .and(warp::path::end())
        .and(post_json::<RunStealthRequestApi>())
        .and(emulator_command_filter.clone())
        .and_then(run_stealth);

    let read_memory_f = warp::post()
        .and(warp::path("internal"))
        .and(warp::path("read_memory"))
        .and(warp::path::end())
        .and(post_json::<ReadMemoryRequestApi>())
        .and(emulator_command_filter.clone())
        .and_then(read_memory);
    

    load_game_f
    .or(load_save_f)
    .or(unload_game_f)
    .or(resume_f)
    .or(input_f)
    .or(get_screen_data_f)
    .or(safe_f)
    .or(register_audio_queue_f)
    .or(get_audio_samples_f)
    .or(run_stealth_f)
    .or(read_memory_f)
    .boxed()
}

