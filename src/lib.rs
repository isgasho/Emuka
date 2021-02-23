#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

use std::{path::PathBuf};

use emulators::{Emulator, EmulatorCommand};
use game::{GameFromFile, SaveFile};
use tokio::{sync::mpsc::{UnboundedSender, unbounded_channel}, time};

pub mod emulators;
pub mod game;
pub mod audio;
pub mod server;




