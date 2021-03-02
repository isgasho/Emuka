pub mod api;
use crate::{audio::VecStereoWrapper, emulators::ScreenData, server::api::v1::api::*};

use std::{collections::VecDeque, convert::TryInto};

use color_eyre::Report;
use tokio::sync::oneshot;
use warp::{Filter, Reply, filters::BoxedFilter};
use uuid::Uuid;

use crate::{audio::{SAMPLE_RATE, StereoSample}, emulators::{EmulatorCommand, EmulatorJoypadInput}, game::{GameFromFile, SaveFile}};
use crate::audio::SAMPLES_MAP;


use super::{CommandSender};

async fn load_game(
    game_api: GameFromFileApi,
    sender: CommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    let result: Result<GameFromFile, Report> = game_api.try_into();

    let reply = match result {
        Ok(game) => {
            sender.send_command(EmulatorCommand::LoadGame(Box::from(game)));
            Ok(warp::reply::with_status(warp::reply(), warp::http::StatusCode::OK))
        },
        Err(err) => {
            eprintln!("{}", err);
            Ok(warp::reply::with_status(warp::reply(), warp::http::StatusCode::BAD_REQUEST))
        }
    };
    reply
}

async fn load_save(
    save_api: SaveFromFileApi,
    sender: CommandSender
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
    sender: CommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    sender.send_command(EmulatorCommand::Resume);

    Ok(warp::reply())
}

async fn input(
    input_api: EmulatorJoypadInputApi,
    sender: CommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    let input: (EmulatorJoypadInput, bool) = input_api.into();
    sender.send_command(EmulatorCommand::Input(input));
    Ok(warp::reply())
}

async fn get_screen_data(
    sender: CommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    let (os_sender, os_receiver) = oneshot::channel::<Option<ScreenData>>();
    sender.send_command(EmulatorCommand::GetScreenData(os_sender));
    let value = os_receiver.await.unwrap();
    let data = ScreenDataApi::from(value);
    Ok(warp::reply::json(&data))
}

async fn save(
    sender: CommandSender
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

    match data {
        Some(samples) => {
            let wrapper = VecStereoWrapper {
                inner: Some(samples)
            };
            let encoded: String = wrapper.into();
            let res = GetAudioSamplesResponseApi {
                data: Some(encoded)
            };
        
            Ok(warp::reply::json(&res))
        }
        None => {
            let res = GetAudioSamplesResponseApi {
                data: None
            };
        
            Ok(warp::reply::json(&res))
        }
    }
}


pub fn routes(sender: CommandSender) -> BoxedFilter<(impl Reply,)> {
    let command_filter = warp::any().map(move || sender.clone());
    
    let load_game_f = warp::post()
        .and(warp::path("game"))
        .and(warp::path("load"))
        .and(warp::path::end())
        .and(post_json::<GameFromFileApi>())
        .and(command_filter.clone())
        .and_then(load_game);
    

    let load_save_f = warp::post()
        .and(warp::path("save"))
        .and(warp::path("load"))
        .and(warp::path::end())
        .and(post_json::<SaveFromFileApi>())
        .and(command_filter.clone())
        .and_then(load_save);

    let resume_f = warp::get()
        .and(warp::path("resume"))
        .and(warp::path::end())
        .and(command_filter.clone())
        .and_then(resume);

    let input_f = warp::post()
        .and(warp::path("input"))
        .and(warp::path::end())
        .and(post_json::<EmulatorJoypadInputApi>())
        .and(command_filter.clone())
        .and_then(input);

    let get_screen_data_f = warp::get()
        .and(warp::path("screen"))
        .and(warp::path::end())
        .and(command_filter.clone())
        .and_then(get_screen_data);

    let safe_f = warp::get()
        .and(warp::path("save"))
        .and(warp::path("save"))
        .and(warp::path::end())
        .and(command_filter.clone())
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

    load_game_f
    .or(load_save_f)
    .or(resume_f)
    .or(input_f)
    .or(get_screen_data_f)
    .or(safe_f)
    .or(register_audio_queue_f)
    .or(get_audio_samples_f)
    .boxed()
}

