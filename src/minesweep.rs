use iced::{Application, Theme, executor, widget::{row, column, button, text, container, canvas::{self, Cache, Path, Event, Cursor, event}, Canvas, Column, Row, Button}, Element, Alignment, theme, Length, Vector, Point, Color, Size, Rectangle};

use minefield_rs::Minefield;

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    Info,
    Settingss,
}

pub struct Minesweep {
    field: Minefield,
    field_cache: Cache,
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
                field_cache: Cache::default(),
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
        info!("{:?}", message);
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let content = column![
            self.view_controls(),
            self.view_field()
        ]
        .align_items(Alignment::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl Minesweep {
    const APP_NAME: &str = "Minesweep-Rs - Iced";
    const REFRESH_BTN_CHAR: &str = "New";
    const SETTINGS_BTN_CHAR: &str = "Settings";
    const ABOUT_BTN_CHAR: &str = "About";
    const SPOT_SIZE: f32 = 40.0;
    const SPOT_PAD: f32 = 5.0;

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

    fn view_field(&self) -> Element<Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

impl canvas::Program<Message> for Minesweep {
    type State = Interaction;

    fn update(
        &self,
        interaction: &mut Interaction,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        let cursor_position =
            if let Some(position) = cursor.position_in(&bounds) {
                position
            } else {
                return (event::Status::Ignored, None);
            };
        
        match event {
            Event::Mouse(m) => {
               // TODO: add handling for mouse (desktop + WASM in browser)
               (event::Status::Ignored, None)
            },
            Event::Touch(_t) => {
                // TODO: add handling for touch (WASM on mobile devices)
                (event::Status::Ignored, None)
            },
            Event::Keyboard(_) => (event::Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        state: &Interaction,
        theme: &Theme,
        bounds: iced::Rectangle,
        cursor: canvas::Cursor,
    ) -> Vec<canvas::Geometry> {
        let field = self.field_cache.draw(bounds.size(), |frame| {
            // Set the background
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            frame.fill(&background, Color::from_rgb8(0x40, 0x44, 0x4B));

            // determine where to draw the spots
            let row_size = self.field.width() as f32 * Self::SPOT_SIZE + (self.field.width().saturating_sub(1) as f32 * Self::SPOT_PAD);
            let col_size = self.field.height() as f32 * Self::SPOT_SIZE + (self.field.height().saturating_sub(1) as f32 * Self::SPOT_PAD);

            let o_x = (frame.width() - row_size) / 2.0;
            let o_y = (frame.height() - col_size) / 2.0;

            // draw the spots
            for y in 0..self.field.height() {
                for x in 0..self.field.width() {
                    if let Some(spot) = self.field.spot(x, y) {
                        let x = o_x + (x as f32 * (Self::SPOT_SIZE + Self::SPOT_PAD));
                        let y = o_y + (y as f32 * (Self::SPOT_SIZE + Self::SPOT_PAD));

                        frame.fill_rectangle(
                            Point::new( x, y),
                            Size::new(Self::SPOT_SIZE, Self::SPOT_SIZE),
                            Color::WHITE,
                        );
                    }
                }
            }
        });

        vec![field]
    }
}

pub enum Interaction {
    None,
    Drawing,
    Erasing,
    Panning { translation: Vector, start: Point },
}

impl Default for Interaction {
    fn default() -> Self {
        Self::None
    }
}