use std::{path::Path, fs::File, io::{Read, Write}};

use iced::{Settings, Application, window};
use minesweep::{Minesweep, GamePersistence};

#[macro_use]
extern crate log;

mod minesweep;

pub fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    let path = "minesweep-rs.json";

    let game_persistence = load_game_persistence(path);

    let res = Minesweep::run(Settings {
        antialiasing: true,
        window: window::Settings {
            position: window::Position::Centered,
            resizable: false,
            ..window::Settings::default()
        },
        flags: game_persistence.clone(),
        ..Settings::default()
    });

    // // DEBUG: 
    // let game_persistence = GamePersistence::default();

    // save_game_persistence(path, game_persistence);

    res
}

fn load_game_persistence<P: AsRef<Path>>(path: P) -> Option<GamePersistence> {
    if let Ok(mut file) = File::open(path) {
        let mut buf = vec![];
        if file.read_to_end(&mut buf).is_ok() {
            if let Ok(world) = serde_json::from_slice(&buf[..]) {
                return Some(world);
            }
        }
    }

    None
}

fn save_game_persistence<P: AsRef<Path>>(path: P, world: GamePersistence) {
    let mut f = File::create(path).unwrap();
    let buf = serde_json::to_vec(&world).unwrap();
    f.write_all(&buf[..]);
}