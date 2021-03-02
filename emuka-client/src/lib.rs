mod audio;
mod server;

use eyre::Result;

pub async fn init() -> Result<()> {
    let stream = audio::init_audio_stream();
    audio::init_audio_requests(String::from("http://localhost:3030")).await?;
    server::init().await;
    
    return Ok(());
}
