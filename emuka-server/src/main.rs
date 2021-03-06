use color_eyre::eyre::Result;
#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let audio_sender = emuka_server::audio::init();
    let emulator_sender = emuka_server::emulators::init().await;
    emuka_server::server::init(emulator_sender, audio_sender).await;
    
    return Ok(());
}

