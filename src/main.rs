use color_eyre::eyre::Result;
use emuka::emulators;

fn main() -> Result<()> {
    color_eyre::install()?;

    println!("I'm Emuka!");

    unsafe {
        crate::emulators::sameboy::bindings::emuka_testprint();
    }
    
    return Ok(());
}

