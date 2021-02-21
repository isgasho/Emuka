# Erika Emulator
> Or Emuka, for short

Wraps an emulator backend (currently only Sameboy, but others could be added) as a headless webserver, and allows it to be queried for its state.

Still stream the audio to the OS's native interfaces, but does not plan to provide any kind of GUI, and it must instead be implemented separately.

Currently in very, very alpha.

## Build

You first need to recursively clone this project, to pull the emulators' (Sameboy at the moment) sources.  
Requires the latest rust toolchain, as well as make and clang for Sameboy. Also requires anything Sameboy requires for Windows compilation, minus the SDL2 library.  
Should build out of the box on Linux with a `cargo build`, not currently tested on Windows (but support will be provided later), macOS is a target I'm looking for but I lack any devide to run it on.  

## Run

`cargo run` will attempt to find a file called "./game/lsdj.gb" that you must provide, and will also attempt to connect to your default stereo interface (in a very unreliable manner, admitedly).  
Provided that both works, you should hear the gameboy color boot sound!
After a minute or two, a save should be generated at "./game/lsdj.sav"!
