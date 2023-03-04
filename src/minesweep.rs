use iced::{Application, Theme, executor, widget::{row, column, button, text, container}, Element, Alignment, theme, Length};

use crate::minefield::Minefield;



#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    Info,
    Settingss,
    None
}

pub struct Minesweep {
    field: Minefield,
    seconds: Option<u16>,
    flags: Option<i64>
}

impl Application for Minesweep {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                field: Minefield::new(10, 10),
                seconds: None,
                flags: None,
            },
            iced::Command::none()
        )
    }

    fn title(&self) -> String {
        String::from(Self::APP_NAME)
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        container(self.view_controls())
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Minesweep {
    const APP_NAME: &str = "Minesweep-Rs - Iced";
    const REFRESH_BTN_CHAR: &str = "New";
    const SETTINGS_BTN_CHAR: &str = "Settngs";
    const ABOUT_BTN_CHAR: &str = "About";

    fn view_controls(&self) -> Element<Message> {
        let display_seconds = column![
            text("Time").size(10),
            if let Some(s) = self.seconds {
                text(s).size(50)
            } else {
                text("---").size(50)
            }
        ]
        .align_items(Alignment::Center);

        let display_flags = column![
            text("Flags").size(10),
            if let Some(f) = self.flags {
                text(f).size(50)
            } else {
                text("---").size(50)
            }
        ]
        .align_items(Alignment::Center);

        row![
            button(Self::REFRESH_BTN_CHAR)
                .on_press(Message::Reset)
                .style(theme::Button::Primary),
            display_seconds,
            display_flags,
            button(Self::SETTINGS_BTN_CHAR)
                .on_press(Message::Settingss)
                .style(theme::Button::Primary),
            button(Self::ABOUT_BTN_CHAR)
                .on_press(Message::Info)
                .style(theme::Button::Primary),
        ]
        .padding(10)
        .spacing(20)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        .into()
    }
}