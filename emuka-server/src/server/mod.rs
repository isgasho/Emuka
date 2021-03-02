pub mod api;

use tokio::sync::mpsc::UnboundedSender;
use crate::emulators::EmulatorCommand;
use warp::{Filter, hyper::Method};

pub async fn init(sender: UnboundedSender<EmulatorCommand>) {
    // env_logger::init();
    // let log = warp::log("emuka");

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers", "Content-Type"])
        .allow_methods(&[Method::GET, Method::POST]);

    let routes = warp::path("api")
        .and(api::routes(sender))
        .with(cors);
        // .with(log); 

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
