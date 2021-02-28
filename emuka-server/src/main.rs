use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let _stream = emuka_server::audio::init_audio(); 
    let sender = emuka_server::emulators::init().await;
    emuka_server::server::init(sender).await;
    
    return Ok(());
}

