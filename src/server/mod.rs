mod api;

use tokio::sync::mpsc::UnboundedSender;
use crate::emulators::EmulatorCommand;
use warp::Filter;

pub async fn init(sender: UnboundedSender<EmulatorCommand>) {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST"]);
    
    let routes = warp::path("api")
        .and(api::routes(sender))
        .with(cors);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
