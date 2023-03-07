use iced::{Application, Theme, executor, widget::{self, canvas::{self, Cache, Path, Event, Cursor, event, Text}, Canvas}, Element, Alignment, theme, Length, Vector, Point, Color, Size, Rectangle, alignment, Command};
use iced_native::{command, window};
use minefield_rs::{Minefield, StepResult, FlagToggleResult};

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
    
    elapsed_seconds: Option<u32>,
    remaining_flags: i64,

    game_config: GameConfig,
}

impl Application for Minesweep {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        // TODO: load game config if available
        let game_config = GameConfig::default();

        let minesweep = Self {
            field: Minefield::new(game_config.width, game_config.height).with_mines(game_config.mines),
            field_cache: Cache::default(),
            game_state: GameState::default(),
            game_config,
            elapsed_seconds: None,
            remaining_flags: game_config.mines as i64,
        };
        let (width, height) = minesweep.desired_window_size();

        let command = Command::single(command::Action::Window(window::Action::Resize { width, height }));

        (minesweep, command)
    }

    fn title(&self) -> String {
        String::from(Self::APP_NAME)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Minesweep { message } => {
                match message {
                    MinesweepMessage::Step { x, y } => {
                        self.check_ready_to_running();
                        
                        if let GameState::Running =self.game_state {
                            let step_result = self.field.step(x, y);

                            match step_result {
                                StepResult::Boom => {
                                    self.game_over(false);
                                },
                                StepResult::Phew => {
                                    if self.field.is_cleared() {
                                        self.game_over(true);
                                    }
                                },
                                _ => {},
                            }
                        }
                    },
                    MinesweepMessage::AutoStep { x, y } => {
                        if let GameState::Running =self.game_state {
                            match self.field.auto_step(x, y) {
                                StepResult::Boom => {
                                    self.game_over(false);
                                },
                                StepResult::Phew => {
                                    if self.field.is_cleared() {
                                        self.game_over(true);
                                    }
                                },
                                _ => {},
                            }
                        }
                    },
                    MinesweepMessage::Flag { x, y } => {
                        self.check_ready_to_running();

                        if let GameState::Running =self.game_state {
                            match self.field.toggle_flag(x, y) {
                                FlagToggleResult::Removed => {
                                    self.remaining_flags += 1;
                                },
                                FlagToggleResult::Added => {
                                    self.remaining_flags -= 1;
                                    
                                    if self.field.is_cleared() {
                                        self.game_over(true);
                                    }
                                },
                                _ => {},
                            }
                        }
                    },
                }
                
                self.field_cache.clear();
                Command::none()
            },
            Message::Reset => {
                self.field = Minefield::new(self.game_config.width, self.game_config.height).with_mines(self.game_config.mines);
                
                self.game_state = GameState::Ready;
                self.elapsed_seconds = None;
                self.remaining_flags = self.game_config.mines as i64;

                self.field_cache.clear();
                
                Command::none()
            },
            Message::Info => {
                // Testing code
                self.game_config = GameDifficulty::MEDIUM;

                self.field = Minefield::new(self.game_config.width, self.game_config.height).with_mines(self.game_config.mines);
                self.game_state = GameState::default();
                self.elapsed_seconds = None;
                self.remaining_flags = self.game_config.mines as i64;
                
                let (width, height) = self.desired_window_size();
        
                let command = Command::single(command::Action::Window(window::Action::Resize { width, height }));
        
                command
            },
            Message::Settings => {
                // Testing code
                self.game_config = GameDifficulty::HARD;

                self.field = Minefield::new(self.game_config.width, self.game_config.height).with_mines(self.game_config.mines);
                self.game_state = GameState::default();
                self.elapsed_seconds = None;
                self.remaining_flags = self.game_config.mines as i64;
                
                let (width, height) = self.desired_window_size();
        
                let command = Command::single(command::Action::Window(window::Action::Resize { width, height }));
        
                command
            },
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let content = widget::column![
            self.view_controls(),
            self.view_field()
        ]
        .align_items(Alignment::Start);

        widget::container(content)
            .width(Length::Shrink)
            .height(Length::Shrink)
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
    
    const TOOLBAR_HEIGHT: f32 = 100.0;
    const FIELD_PAD: f32 = 20.0;
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
    // const MINE_EXPLODED_CHAR: &str = "💥";
    const MINE_EXPLODED_CHAR: &str = "*";
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

    fn desired_window_size(&self) -> (u32, u32) {
        let (field_width, field_height) = self.desired_field_size();
        
        let width = field_width as u32;
        let height = field_height as u32 + Self::TOOLBAR_HEIGHT as u32;

        (width, height)
    }

    fn desired_field_size(&self) -> (f32, f32) {
        let width = (Self::SPOT_SIZE * self.field.width() as f32) + (Self::FIELD_PAD * 2.0);
        let height = (Self::SPOT_SIZE * self.field.height() as f32) + (Self::FIELD_PAD * 2.0);

        (width, height)
    }

    fn view_controls(&self) -> Element<Message> {
        
        let time_text = match self.game_state {
            GameState::Ready => {
                widget::text("---").size(50)
            },
            GameState::Running => {
                widget::text(self.elapsed_seconds.unwrap()).size(50)
            },
            GameState::Stopped { is_won: _ } => {
                widget::text(self.elapsed_seconds.unwrap()).size(50)
            },
        };
        let display_seconds = widget::column![
            widget::text("Time").size(10),
            time_text
        ]
        .align_items(Alignment::Center);

        let flags_text = match self.game_state {
            GameState::Ready => {
                widget::text("---").size(50)
            },
            GameState::Running => {
                widget::text(self.remaining_flags).size(50)
            },
            GameState::Stopped { is_won: _} => {
                widget::text(self.remaining_flags).size(50)
            },
        };
        let display_flags = widget::column![
            widget::text("Flags").size(10),
            flags_text
        ]
        .align_items(Alignment::Center);

        widget::row![
            widget::button(Self::REFRESH_BTN_CHAR)
                .on_press(Message::Reset)
                .style(theme::Button::Primary),
            display_seconds,
            display_flags,
            widget::button(Self::SETTINGS_BTN_CHAR)
                .on_press(Message::Settings)
                .style(theme::Button::Primary),
            widget::button(Self::ABOUT_BTN_CHAR)
                .on_press(Message::Info)
                .style(theme::Button::Primary),
        ]
        .padding(10)
        .spacing(20)
        .align_items(Alignment::Start)
        .width(Length::Fill)
        .into()
    }

    fn view_field(&self) -> Element<Message> {
        let (field_width, field_height) = self.desired_field_size();
        Canvas::new(self)
            .width(field_width)
            .height(field_height)
            .into()
    }

    fn check_ready_to_running(&mut self) {
        if let GameState::Ready = self.game_state {
            self.game_state = GameState::Running;
            self.elapsed_seconds = Some(0);
            
            // TODO: start timer
        }
    }
    
    fn game_over(&mut self, is_won: bool) {
        self.game_state = GameState::Stopped{is_won};
        // TODO: stop timer
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
        _state: &Interaction,
        _theme: &Theme,
        bounds: iced::Rectangle,
        _cursor: canvas::Cursor,
    ) -> Vec<canvas::Geometry> {
        let field = self.field_cache.draw(bounds.size(), |frame| {
            // Set the background
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            let background_color = Color::from_rgb8(0x40, 0x44, 0x4B);
            frame.fill(&background, background_color);

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
                    minefield_rs::SpotState::HiddenEmpty { neighboring_mines: _ } => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            foreground_color,
                        );
                    },
                    minefield_rs::SpotState::HiddenMine => {
                        frame.fill_rectangle(
                            p,
                            Size::new(Self::CELL_SIZE, Self::CELL_SIZE),
                            foreground_color,
                        );
                    },
                    minefield_rs::SpotState::FlaggedEmpty { neighboring_mines: _ } | minefield_rs::SpotState::FlaggedMine => {
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
    Running,

    /// Game is stopped, and was either won (`true`), or lost (`false`)
    Stopped { is_won: bool }
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