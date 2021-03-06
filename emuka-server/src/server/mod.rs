pub mod api;

use std::sync::mpsc::Sender;

use tokio::sync::mpsc::UnboundedSender;
use crate::{audio::AudioCommand, emulators::EmulatorCommand};
use warp::{Filter, hyper::Method};

pub async fn init(emulator_sender: UnboundedSender<EmulatorCommand>, audio_sender: Sender<AudioCommand>) {
    // env_logger::init();
    // let log = warp::log("emuka");

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers", "Content-Type"])
        .allow_methods(&[Method::GET, Method::POST]);

    let routes = warp::path("api")
        .and(api::routes(emulator_sender, audio_sender))
        .with(cors);
        // .with(log); 

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
