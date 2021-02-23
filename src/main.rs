use color_eyre::eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    emuka::init().await;
    
    return Ok(());
}

