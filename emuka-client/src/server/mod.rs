use reqwest::Method;
use warp::Filter;

pub async fn init() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers", "Content-Type"])
        .allow_methods(&[Method::GET, Method::POST]);

    let routes = warp::any().map(|| "nya");

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3031))
        .await;
}
