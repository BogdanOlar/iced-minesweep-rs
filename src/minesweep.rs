use iced::{Application, Theme, executor, widget::{row, column, button, text, container, canvas::{self, Cache, Path, Event, Cursor, event, Text}, Canvas, Column, Row, Button}, Element, Alignment, theme, Length, Vector, Point, Color, Size, Rectangle, alignment};

use minefield_rs::Minefield;

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    Info,
    Settings,
    Minesweep { message: MinesweepMessage },
}

/// Lower level game logic messages
#[derive(Debug, Clone)]
pub enum MinesweepMessage {
    Step{x: u16, y: u16},
    AutoStep{x: u16, y: u16},
    Flag{x: u16, y: u16},
}

pub struct Minesweep {
    /// Model
    field: Minefield,

    /// View: a cache of the canvas holding the minefield. A redraw can be forced on it by calling `field_cache.clear()`
    field_cache: Cache,

    game_state: GameState,

    game_config: GameConfig,

    seconds: Option<u16>,
    flags: Option<i64>
}

impl Application for Minesweep {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        // TODO: load game config if available
        let game_config = GameConfig::default();

        (
            Self {
                field: Minefield::new(game_config.width, game_config.height).with_mines(game_config.mines),
                field_cache: Cache::default(),
                game_state: GameState::default(),
                game_config,
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
            Message::Minesweep { message } => {
                match message {
                    MinesweepMessage::Step { x, y } => {
                        let _step_result = self.field.step(x, y);
                        
                        self.field_cache.clear();
        
                        iced::Command::none()
                    },
                    MinesweepMessage::AutoStep { x, y } => {
                        let _auto_step_result = self.field.auto_step(x, y);
                        
                        self.field_cache.clear();
        
                        iced::Command::none()
                    },
                    MinesweepMessage::Flag { x, y } => {
                        let _flag_result = self.field.toggle_flag(x, y);
                        
                        self.field_cache.clear();
        
                        iced::Command::none()
                    },
                }
            },
            Message::Reset => {
                iced::Command::none()
            },
            Message::Info => {
                iced::Command::none()
            },
            Message::Settings => {
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

    const MINE_CHAR: &str = "☢";
    const MINE_COLOR: Color = Self::COLOR_RED;
    const MINE_EXPLODED_CHAR: &str = "💥";
    const MINE_EXPLODED_COLOR: Color = Self::COLOR_RED;
    // const FLAG_CHAR: &str = "⚐";
    const FLAG_CHAR: &str = "f";
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

    #[allow(dead_code)]
    pub fn with_configs(mut self, game_config: GameConfig) -> Self {
        self.game_config = game_config;
        self.field = Minefield::new(self.game_config.width, self.game_config.height).with_mines(self.game_config.mines);

        self
    }

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
                .on_press(Message::Settings)
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
                                    (event::Status::Captured, Some(Message::Minesweep { message: MinesweepMessage::Step { x, y } }))
                                },
                                iced::mouse::Button::Right => {
                                    (event::Status::Captured, Some(Message::Minesweep { message: MinesweepMessage::Flag { x, y } }))
                                },
                                iced::mouse::Button::Middle => {
                                    (event::Status::Captured, Some(Message::Minesweep { message: MinesweepMessage::AutoStep { x, y } }))
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
                        frame.fill_text(Text {
                            content: format!("{}", Self::FLAG_CHAR),
                            position: text.position,
                            color: Self::FLAG_COLOR_CORRECT,
                            ..text
                        });
                    },
                    minefield_rs::SpotState::FlaggedMine => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            foreground_color,
                        );
                        frame.fill_text(Text {
                            content: format!("{}", Self::FLAG_CHAR),
                            position: text.position,
                            color: Self::FLAG_COLOR_CORRECT,
                            ..text
                        });
                    },
                    minefield_rs::SpotState::RevealedEmpty { neighboring_mines } => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            background_color,
                        );
                        
                        frame.fill_text(Text {
                            content: format!("{}", Self::EMPTY_SPOT_CHARS[neighboring_mines as usize]),
                            position: text.position,
                            color: Self::EMPTY_SPOT_COLORS[neighboring_mines as usize],
                            ..text
                        });
                    },
                    minefield_rs::SpotState::ExplodedMine => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            foreground_color,
                        );
                        frame.fill_text(Text {
                            content: format!("{}", Self::MINE_EXPLODED_CHAR),
                            position: text.position,
                            color: Self::MINE_EXPLODED_COLOR,
                            ..text
                        });
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

/// Current state of the game
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum GameState {
    /// Game is ready to start running
    Ready,

    /// Game is running
    Running { seconds: u32, flags_placed: u32 },

    /// Game is stopped, and was either won (`true`), or lost (`false`)
    Stopped { is_won: bool, seconds: u32, flags_placed: u32}
}

impl Default for GameState {
    fn default() -> Self {
        Self::Ready
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub mines: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { width: 10, height: 10, mines: 10 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameDifficulty {
    Easy,
    Medium,
    Hard,
}

impl GameDifficulty {
    pub const EASY: GameConfig = GameConfig { width: 10, height: 10, mines: 10 };
    pub const MEDIUM: GameConfig = GameConfig { width: 16, height: 16, mines: 40 };
    pub const HARD: GameConfig = GameConfig { width: 30, height: 16, mines: 99 };

    pub fn from_config(config: &GameConfig) -> Self {
        if *config == Self::EASY {
            Self::Easy
        } else if *config == Self::MEDIUM {
            Self::Medium
        } else if *config == Self::HARD {
            Self::Hard
        } else {
            unreachable!()
        }
    }
}