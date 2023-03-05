use iced::{Application, Theme, executor, widget::{row, column, button, text, container, canvas::{self, Cache, Path, Event, Cursor, event, Text}, Canvas, Column, Row, Button}, Element, Alignment, theme, Length, Vector, Point, Color, Size, Rectangle, alignment};

use minefield_rs::Minefield;

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    Info,
    Settingss,
    Step{x: u16, y: u16},
    AutoStep{x: u16, y: u16},
    Flag{x: u16, y: u16}
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
                field: Minefield::new(10, 5).with_mines(3),
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
        match message {
            Message::Step { x, y } => {
                let step_result = self.field.step(x, y);
                
                dbg!(step_result);
                self.field_cache.clear();

                iced::Command::none()
            },
            Message::AutoStep { x, y } => {
                let step_result = self.field.auto_step(x, y);
                
                dbg!(step_result);
                self.field_cache.clear();

                iced::Command::none()
            },
            Message::Flag { x, y } => {
                let flag_result = self.field.toggle_flag(x, y);
                
                dbg!(flag_result);
                self.field_cache.clear();

                iced::Command::none()
            },
            Message::Reset => {
                iced::Command::none()
            },
            Message::Info => {
                iced::Command::none()
            },
            Message::Settingss => {
                iced::Command::none()
            },
        }
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
    const APP_NAME: &str = "iced minesweep-rs";
    const REFRESH_BTN_CHAR: &str = "New";
    const SETTINGS_BTN_CHAR: &str = "Settings";
    const ABOUT_BTN_CHAR: &str = "About";
    // const REFRESH_BTN_CHAR: &str = "🔄";
    // const SETTINGS_BTN_CHAR: &str = "🛠";
    // const ABOUT_BTN_CHAR: &str = "ℹ";
    
    /// Size of spor on canvas, including padding
    const SPOT_SIZE: f32 = 40.0;
    /// Interior padding of spot
    const SPOT_PAD: f32 = 5.0;
    const CELL_SIZE: f32 = Self::SPOT_SIZE - (Self::SPOT_PAD * 2.0);


    const COLOR_RED: Color = Color::from_rgb(255.0, 0.0, 0.0);
    const COLOR_LIGHT_RED: Color = Color::from_rgb(255.0, 128.0, 128.0);
    const COLOR_GREEN: Color = Color::from_rgb(0.0, 255.0, 0.0);
    const COLOR_GRAY: Color = Color::from_rgb(160.0, 160.0, 160.0);

    const MINE_CAHR: &str = "☢";
    const MINE_COLOR: Color = Self::COLOR_RED;
    const MINE_EXPLODED_CHAR: &str = "💥";
    const MINE_EPLODED_COLOR: Color = Self::COLOR_RED;
    const FLAG_CHAR: &str = "⚐";
    const FLAG_COLOR_CORRECT: Color = Self::COLOR_GREEN;
    const FLAG_COLOR_WRONG: Color = Self::COLOR_RED;
    const EMPTY_SPOT_CHARS: [&str; 9] = [" ", "1", "2", "3", "4", "5", "6", "7", "8"];
    const EMPTY_SPOT_COLORS: [Color; Self::EMPTY_SPOT_CHARS.len()] = [
        Color::WHITE, Color::WHITE, Color::WHITE,
        Color::WHITE, Color::WHITE, Color::WHITE,
        Color::WHITE, Color::WHITE, Color::WHITE
    ];
    const HIDDEN_SPOT_CHAR: &str = " ";
    const HIDDEN_SPOT_COLOR: Color = Self::COLOR_GRAY;
    const WON_COLOR: Color = Self::COLOR_GREEN;
    const LOST_COLOR: Color = Self::COLOR_RED;
    const READY_COLOR: Color = Self::COLOR_GRAY;
    const FLAG_COUNT_OK_COLOR: Color = Self::COLOR_GRAY;
    const FLAG_COUNT_ERR_COLOR: Color = Self::COLOR_LIGHT_RED;

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
        // determine where to draw the spots
        let f_width = self.field.width() as f32 * Self::SPOT_SIZE;
        let f_height = self.field.height() as f32 * Self::SPOT_SIZE;

        let f_o_x = (bounds.width - f_width) / 2.0;
        let f_o_y = (bounds.height - f_height) / 2.0;
        let origin_point = Point::new( bounds.x + f_o_x, bounds.y + f_o_y);
        let origin_rectangle = Rectangle::new(origin_point, Size::new(f_width, f_height));

        if let Some(position) =  cursor.position_in(&origin_rectangle) {
            let x = (position.x / Self::SPOT_SIZE as f32).floor() as u16;
            let y = (position.y / Self::SPOT_SIZE as f32).floor() as u16;

            match event {
                Event::Mouse(mouse_event) => {
                    match mouse_event {
                        iced::mouse::Event::ButtonPressed(mouse_button) => {
                            match mouse_button {
                                iced::mouse::Button::Left => {
                                    (event::Status::Captured, Some(Message::Step { x, y }))
                                },
                                iced::mouse::Button::Right => {
                                    (event::Status::Captured, Some(Message::Flag { x, y }))
                                },
                                iced::mouse::Button::Middle => {
                                    (event::Status::Captured, Some(Message::AutoStep { x, y }))
                                },
                                iced::mouse::Button::Other(_) => {
                                    (event::Status::Ignored, None)
                                },
                            }
                        },
                        _ => {
                            (event::Status::Ignored, None)
                        }
                    }
                },
                Event::Touch(_t) => {
                    // TODO: add handling for touch (WASM on mobile devices)
                    (event::Status::Ignored, None)
                },
                Event::Keyboard(_) => (event::Status::Ignored, None),
            }
        } else {
            (event::Status::Ignored, None)
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
            let background_color = Color::from_rgb8(0x40, 0x44, 0x4B);
            frame.fill(&background, background_color.clone());

            // determine where to draw the spots
            let f_width = self.field.width() as f32 * Self::SPOT_SIZE;
            let f_height = self.field.height() as f32 * Self::SPOT_SIZE;

            let f_o_x = (frame.width() - f_width) / 2.0;
            let f_o_y = (frame.height() - f_height) / 2.0;
            let origin_point = Point::new( f_o_x, f_o_y);

            let foreground_color = Color::WHITE;
            // draw the spots
            for (&(ix, iy), spot) in self.field.spots() {
                let fx = (ix as f32 * Self::SPOT_SIZE) + Self::SPOT_PAD;
                let fy = (iy as f32 * Self::SPOT_SIZE) + Self::SPOT_PAD;
                let p = origin_point + Vector::new(fx, fy);

                let text = Text {
                    color: Color::from_rgb8(0xAA, 0x47, 0x8A),
                    size: Self::CELL_SIZE,
                    position: p,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    ..Text::default()
                };
                
                match spot.state {
                    minefield_rs::SpotState::HiddenEmpty { neighboring_mines } => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            foreground_color,
                        );
                        frame.fill_text(Text {
                            content: format!("{}", neighboring_mines),
                            position: text.position,
                            ..text
                        });
                    },
                    minefield_rs::SpotState::HiddenMine => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            foreground_color,
                        );
                        frame.fill_text(".");
                    },
                    minefield_rs::SpotState::FlaggedEmpty { neighboring_mines: _ } => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            foreground_color,
                        );
                        frame.fill_text("F");
                    },
                    minefield_rs::SpotState::FlaggedMine => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            foreground_color,
                        );
                        frame.fill_text("F");
                    },
                    minefield_rs::SpotState::RevealedEmpty { neighboring_mines } => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            background_color,
                        );
                        frame.fill_text(format!("{}",neighboring_mines));
                    },
                    minefield_rs::SpotState::ExplodedMine => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            foreground_color,
                        );
                        frame.fill_text("X");
                    },
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