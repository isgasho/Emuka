use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let _stream = emuka::audio::init_audio(); 
    let sender = emuka::emulators::init().await;
    emuka::server::init(sender).await;
    
    return Ok(());
}

