use iced::{window, Application, Settings};
use minesweep::Minesweep;

extern crate log;

mod minesweep;

pub fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    Minesweep::run(Settings {
        antialiasing: true,
        window: window::Settings {
            position: window::Position::Centered,
            resizable: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
