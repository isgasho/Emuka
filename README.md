# Erika Emulator
> Or Emuka, for short

Wraps an emulator backend(currently only [SameBoy](https://github.com/LIJI32/SameBoy)) as a headless webserver, and allows it to be queried for its state or to interact with it.

Streams the audio to the OS's native interfaces, but does not plan to provide any kind of GUI, and it must instead be implemented separately by querying the API.

Built for being used by [BSDj](https://github.com/ShinySaana/BSDj). If you think Emuka can be useful for your own project, let's talk!

Emuka is divided in two parts:
- `emuka-server`: the main binary, act as a standalone binary.
- `emuka-client`: remotely connects to a server and forwards requests to it, while still handling the audio part natively. Will be implemented later, and currently exist as a PoC.

Currently in alpha.

## Build

You first need to recursively clone this project, to pull the emulators' (SameBoy at the moment) sources.  

### Within your system (Linux)

#### Linux

Requires the latest rust toolchain, as well as `make` and `clang` for SameBoy.  
Then,

```sh
cargo build --release
```

> Will output the binaries to `./target/release/emuka-[client, server]`

#### Windows

> Only got it to work by cross-compiling from Linux. If you have any idea on how to make this works reliably for Windows, I'm all ears.

Requires the latest rust toolchain, as well as `make` and `mingw64` for SameBoy.

You also need to add Windows as a compilation target for Cargo:

```sh
rustup target add x86_64-pc-windows-gnu
rustup toolchain install stable-x86_64-pc-windows-gnu
```

Then,

```sh
cargo build --release --target x86_64-pc-windows-gnu
```

> Will output the binaries to `./target/x86_64-pc-windows-gnu/release/emuka-[client, server].exe`

### With Docker (on Linux)

Docker builds are provided. Given that your system has a working docker daemon running, you can rust the scripts:

- `./docker/build-linux.sh`
- `./docker/build-windows.sh`

for building the resulting binaries without having to modify your system.

> Will output the binaries to `./docker/out/[linux, windows]/*`

## Run

TODO!
