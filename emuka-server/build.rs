use std::{path::PathBuf, process::{Command}};
use std::env::set_current_dir;
use std::fs::copy;

use bindgen::EnumVariation;

static SAMEBOY_HEADER_PATH: &str = "./libretro/libretro.h";
static SAMEBOY_SOURCE_PATH: &str = "./libretro/libretro.c";
static SAMEBOY_PATH: &str = "./emulators/SameBoy/";

fn main() {
    println!("cargo:rustc-link-search=./lib");
    build_sameboy();
}

fn build_sameboy() {
    println!("cargo:rerun-if-changed=../{}{}", SAMEBOY_PATH, SAMEBOY_HEADER_PATH);
    println!("cargo:rerun-if-changed=../{}{}", SAMEBOY_PATH, SAMEBOY_SOURCE_PATH);
    println!("cargo:rustc-link-lib=static=sameboy");

    set_current_dir(format!("../{}", SAMEBOY_PATH)).unwrap();


    // Fine if it crash
    Command::new("sh")
            .arg("-c")
            .arg("make clean")
            .output();

    let output = if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap() == "windows" {
        Command::new("sh")
            .arg("-c")
            .arg("make PLATFORM=MINGW64 CC=x86_64-w64-mingw32-gcc AR=x86_64-w64-mingw32-ar -C libretro")
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .arg("make CC=clang -C libretro")
            .output()
            .expect("failed to execute process")
    };

    println!("{}{}", String::from_utf8(output.stdout).unwrap(), String::from_utf8(output.stderr).unwrap());

    if !output.status.success() {
        panic!("Err: make failed")
    }

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(SAMEBOY_HEADER_PATH)
        .default_enum_style(EnumVariation::Rust { non_exhaustive: false })
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    set_current_dir("../..").unwrap();


    std::fs::create_dir_all("./lib").unwrap();
    copy(format!("{}./build/bin/sameboy_libretro.a", SAMEBOY_PATH), "./lib/libsameboy.a").unwrap();

    let bindings_path = if std::env::var("CARGO_CFG_TARGET_FAMILY").unwrap() == "windows" {
        "bindings_win.rs"
    } else {
        "bindings_unix.rs"
    };

    let out_path = PathBuf::from(String::from("./emuka-server/src/emulators/sameboy/bindings"));
    bindings
        .write_to_file(out_path.join(bindings_path))
        .expect("Couldn't write bindings!");
}
