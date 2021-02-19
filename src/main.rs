use color_eyre::eyre::Result;
use emuka::emulators::sameboy::init;


fn main() -> Result<()> {
    color_eyre::install()?;

    init();
    
    return Ok(());
}

