use std::{convert::TryInto, process::Command};

use color_eyre::Report;
use warp::{Filter, Rejection, Reply, filters::BoxedFilter, reply::json};

use crate::{emulators::{EmulatorCommand, EmulatorJoypadInput}, game::{self, Game, GameFromFile, SaveFile}};

use super::{CommandSender};

async fn answer_a() -> Result<impl Reply, Rejection> {
    Ok(json(&String::from("A")))
}


async fn answer_b() -> Result<impl Reply, Rejection> {
    Ok(json(&String::from("B")))
}


#[derive(Debug, Deserialize, Clone)]
struct GameFromFileApi {
    path: String
}


impl TryInto<GameFromFile> for GameFromFileApi {
    type Error = eyre::Report;

    fn try_into(self) -> Result<GameFromFile, Self::Error> {
        Ok(GameFromFile::new(&self.path, &self.path)?)
    }
}

fn post_json <T: Send + Sync + serde::de::DeserializeOwned> () -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}


#[derive(Debug, Deserialize, Clone)]
struct SaveFromFileApi {
    path: String
}


impl TryInto<SaveFile> for SaveFromFileApi {
    type Error = eyre::Report;

    fn try_into(self) -> Result<SaveFile, Self::Error> {
        Ok(SaveFile::new(&self.path, &self.path)?)
    }
}

#[derive(Debug, Deserialize, Clone)]
struct EmulatorJoypadInputApi {
    input: EmulatorJoypadInput
}

impl Into<EmulatorJoypadInput> for EmulatorJoypadInputApi {
    fn into(self) -> EmulatorJoypadInput {
        self.input
    }
}



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
    let input: EmulatorJoypadInput = input_api.into();
    sender.send_command(EmulatorCommand::Input(input));
    Ok(warp::reply())
}



pub fn routes(sender: CommandSender) -> BoxedFilter<(impl Reply,)> {
    let command_filter = warp::any().map(move || sender.clone());
    
    let load_game_f = warp::post()
        .and(warp::path("load"))
        .and(warp::path("game"))
        .and(warp::path::end())
        .and(post_json::<GameFromFileApi>())
        .and(command_filter.clone())
        .and_then(load_game);
    

    let load_save_f = warp::post()
        .and(warp::path("load"))
        .and(warp::path("save"))
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

    load_game_f
    .or(load_save_f)
    .or(resume_f)
    .or(input_f)
    .boxed()
}

