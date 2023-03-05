use iced::{Settings, Application, window};
use minesweep::Minesweep;

#[macro_use]
extern crate log;

mod minesweep;

pub fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    Minesweep::run(Settings {
        antialiasing: true,
        window: window::Settings {
            position: window::Position::Centered,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}