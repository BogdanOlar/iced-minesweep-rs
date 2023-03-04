use iced::{Settings, Application, window};
use minesweep::Minesweep;

mod minesweep;
mod minefield;

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