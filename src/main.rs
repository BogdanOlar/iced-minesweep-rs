use iced::{Settings, Application, window};
use minesweep::{Minesweep};

#[macro_use]
extern crate log;

mod minesweep;

pub fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    let res = Minesweep::run(Settings {
        antialiasing: true,
        window: window::Settings {
            position: window::Position::Centered,
            resizable: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    });

    // // DEBUG: 
    // let game_persistence = GamePersistence::default();

    // save_game_persistence(path, game_persistence);

    res
}