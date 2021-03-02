use color_eyre::eyre::Result;

use emuka_client::init;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    init().await?;
    
    return Ok(());
}

