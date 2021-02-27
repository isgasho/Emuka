use std::convert::TryInto;

use color_eyre::Report;
use tokio::sync::oneshot;
use warp::{Filter, Reply, filters::BoxedFilter};

use crate::{emulators::{EmulatorCommand, EmulatorJoypadInput}, game::{GameFromFile, SaveFile}};

use super::{CommandSender};


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
    input: EmulatorJoypadInput,
    pressed: bool
}

impl Into<(EmulatorJoypadInput, bool)> for EmulatorJoypadInputApi {
    fn into(self) -> (EmulatorJoypadInput, bool) {
        (self.input, self.pressed)
    }
}

#[derive(Debug, Serialize, Clone)]
struct ScreenDataApi {
    screen: Option<String>
}

impl From<Option<Vec<u8>>> for ScreenDataApi {
    fn from(data: Option<Vec<u8>>) -> Self {
        Self {
            screen: data.map(
                |bytes| base64::encode(bytes)
            )
        }
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
    let input: (EmulatorJoypadInput, bool) = input_api.into();
    sender.send_command(EmulatorCommand::Input(input));
    Ok(warp::reply())
}

async fn get_screen_data(
    sender: CommandSender
) -> Result<impl warp::Reply, warp::Rejection> {
    let (os_sender, os_receiver) = oneshot::channel::<Option<Vec<u8>>>();
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

    load_game_f
    .or(load_save_f)
    .or(resume_f)
    .or(input_f)
    .or(get_screen_data_f)
    .or(safe_f)
    .boxed()
}

