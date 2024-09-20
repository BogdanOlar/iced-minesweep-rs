use iced::window;
use minesweep::Minesweep;

extern crate log;

mod minesweep;

pub fn main() -> iced::Result {
    env_logger::builder().format_timestamp(None).init();

    iced::application(Minesweep::APP_NAME, Minesweep::update, Minesweep::view)
        .subscription(Minesweep::subscription)
        .font(include_bytes!("../res/fonts/emoji-icon-font.ttf").as_slice())
        .font(include_bytes!("../res/fonts/NotoEmoji-Regular.ttf").as_slice())
        .font(include_bytes!("../res/fonts/Ubuntu-Light.ttf").as_slice())
        .window(window::Settings {
            position: window::Position::Centered,
            resizable: false,
            ..window::Settings::default()
        })
        .run_with(Minesweep::initialize)
}
