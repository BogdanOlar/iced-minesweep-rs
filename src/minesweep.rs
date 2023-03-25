use std::{time::{Duration, Instant}, fmt::Display};
use iced::{
    alignment, executor, mouse, theme, time,
    widget::{self, canvas::{self, event, stroke, Cache, Path, Event, Cursor, Text, Frame, Stroke, LineCap }, Canvas}, 
    Alignment, Application, Color, Command, Element, Length, Point, Rectangle, Size, Subscription, Theme, Vector, Font, 
};
use iced_native::{command, window};
use minefield_rs::{Minefield, StepResult, FlagToggleResult};

#[derive(Debug, Clone)]
pub enum Message {
    Reset,
    Info,
    /// Messages related to game settings
    Settings(SettingsMessage),
    /// Messages related to playing the game
    Minesweep { message: MinesweepMessage },
    Tick(Instant),
}

/// Lower level game logic messages
#[derive(Debug, Clone)]
pub enum MinesweepMessage {
    Step{x: u16, y: u16},
    AutoStep{x: u16, y: u16},
    Flag{x: u16, y: u16},
}

#[derive(Debug, Clone)]
enum SettingsMessage {
    Show,
    Set(GameDifficulty),
    Picked(GameDifficulty),
    ConfigWidth(u16),
    ConfigHeight(u16),
    ConfigMines(u32),
    Discard
}

enum MainViewContent {
    Game,
    Settings(GameDifficulty),
    Info
}

pub struct Minesweep {
    /// Model
    field: Minefield,

    /// View: a cache of the canvas holding the minefield. A redraw can be forced on it by calling `field_cache.clear()`
    field_cache: Cache,

    main_view: MainViewContent,

    game_state: GameState,
    
    elapsed_seconds: Duration,
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
        let game_config = GameDifficulty::EASY;

        let minesweep = Self {
            field: Minefield::new(game_config.width, game_config.height).with_mines(game_config.mines),
            field_cache: Cache::default(),
            main_view: MainViewContent::Game,
            game_state: GameState::default(),
            game_config,
            elapsed_seconds: Duration::default(),
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
                        
                        if let GameState::Running(_) =self.game_state {
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
                        if let GameState::Running(_) =self.game_state {
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

                        if let GameState::Running(_) =self.game_state {
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
                self.elapsed_seconds = Duration::default();
                self.remaining_flags = self.game_config.mines as i64;

                self.field_cache.clear();
                
                Command::none()
            },
            Message::Info => {
                // TODO: add info page (high scores and game info)

                Command::none()
            },
            Message::Settings(settings_message) => {
                match settings_message {
                    SettingsMessage::Show => {
                        match self.main_view {
                            MainViewContent::Settings(_) => {
                                // Get back to the game
                                self.resume_game();
                                self.main_view = MainViewContent::Game;

                                Command::none()
                            },
                            _ => {
                                self.pause_game();
                                self.main_view = MainViewContent::Settings(GameDifficulty::from_config(&self.game_config));
                
                                Command::none()
                            }
                        }
                    },
                    SettingsMessage::Set(game_difficulty) => {
                        self.game_config = game_difficulty.to_config();

                        self.field = Minefield::new(self.game_config.width, self.game_config.height).with_mines(self.game_config.mines);
                        self.game_state = GameState::Ready;
                        self.main_view = MainViewContent::Game;
                        self.elapsed_seconds = Duration::default();
                        self.remaining_flags = self.game_config.mines as i64;
                        
                        let (width, height) = self.desired_window_size();
                        
                        self.field_cache.clear();
                        
                        let command = Command::single(command::Action::Window(window::Action::Resize { width, height }));
                
                        command
                    },
                    SettingsMessage::Picked(gdif) => {
                        self.main_view = MainViewContent::Settings(gdif);

                        Command::none()
                    },
                    SettingsMessage::Discard => {
                        match self.main_view {
                            MainViewContent::Settings(_) => {
                                self.main_view = MainViewContent::Game;
                                self.resume_game();
        
                                Command::none()
                            },
                            _ => {
                                Command::none()
                            }
                        }
                    },
                    SettingsMessage::ConfigWidth(width) => {
                        if let MainViewContent::Settings(game_difficulty) = self.main_view {
                            if let GameDifficulty::Custom(game_config) = game_difficulty {
                                self.main_view = MainViewContent::Settings(GameDifficulty::Custom(
                                    GameConfig { 
                                        width, 
                                        height: game_config.height,
                                        mines: game_config.mines
                                    }
                                ))
                            }
                        }
                        Command::none()
                    },
                    SettingsMessage::ConfigHeight(height) => {
                        if let MainViewContent::Settings(game_difficulty) = self.main_view {
                            if let GameDifficulty::Custom(game_config) = game_difficulty {
                                self.main_view = MainViewContent::Settings(GameDifficulty::Custom(
                                    GameConfig { 
                                        width: game_config.width,
                                        height, 
                                        mines: game_config.mines
                                    }
                                ))
                            }
                        }
                        Command::none()
                    },
                    SettingsMessage::ConfigMines(mines) => {
                        if let MainViewContent::Settings(game_difficulty) = self.main_view {
                            if let GameDifficulty::Custom(game_config) = game_difficulty {
                                self.main_view = MainViewContent::Settings(GameDifficulty::Custom(
                                    GameConfig { 
                                        width: game_config.width,
                                        height: game_config.height,
                                        mines, 
                                    }
                                ))
                            }
                        }
                        Command::none()
                    },
                }
            }

            Message::Tick(new_tick) => {
                if let GameState::Running(cur_tick) = &mut self.game_state {
                    self.elapsed_seconds += new_tick - *cur_tick;
                    *cur_tick = new_tick;
                }
                
                Command::none()
            },
            
            
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let main_view = match self.main_view {
            MainViewContent::Game => {
                self.view_field()
            },
            MainViewContent::Settings(game_difficulty) => {
                self.view_settings(&game_difficulty)
            },
            MainViewContent::Info => todo!(),
        };

        let content = widget::column![
            // TODO: remove commented out debug code
            // self.view_controls().explain(Color::WHITE),
            self.view_controls(),
            main_view
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Start);

        widget::container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
    
    fn subscription(&self) -> Subscription<Message> {
        if let GameState::Running(_) = self.game_state {
            time::every(Duration::from_millis(1000)).map(Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn theme(&self) -> Theme {
        Self::Theme::Dark
    }
}

impl Minesweep {
    const APP_NAME: &str = "iced minesweep-rs";

    // Fonts for mines and flags
    const MINES_FLAGS_ICONS: Font = Font::External {
        name: "Icons",
        bytes: include_bytes!("../res/fonts/emoji-icon-font.ttf"),
    };


    // Fonts for mines and flags
    const COMMANDS_ICONS: Font = Font::External {
        name: "Commands",
        bytes: include_bytes!("../res/fonts/NotoEmoji-Regular.ttf"),
    };

    // Fonts for text
    const TEXT_FONT: Font = Font::External {
        name: "Commands",
        bytes: include_bytes!("../res/fonts/Ubuntu-Light.ttf"),
    };

    const REFRESH_BTN_CHAR: &str = "🔄";
    const SETTINGS_BTN_CHAR: &str = "🛠";
    const ABOUT_BTN_CHAR: &str = "ℹ";
    
    const TOOLBAR_HEIGHT: f32 = 70.0;
    const FIELD_PAD: f32 = 20.0;
    /// Size of spor on canvas, including padding
    const SPOT_SIZE: f32 = 30.0;
    /// Interior padding of spot
    const SPOT_PAD: f32 = 1.0;
    const CELL_SIZE: f32 = Self::SPOT_SIZE - (Self::SPOT_PAD * 2.0);
    const CELL_PAD: f32 = 8.0;


    const COLOR_RED: Color = Color::from_rgb(255.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0);
    const COLOR_LIGHT_RED: Color = Color::from_rgb(255.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0);
    const COLOR_GREEN: Color = Color::from_rgb(0.0 / 255.0, 255.0 / 255.0, 0.0 / 255.0);
    const COLOR_GRAY: Color = Color::from_rgb(60.0 / 255.0, 60.0 / 255.0, 60.0 / 255.0);
    const COLOR_DARK_GRAY: Color = Color::from_rgb(27.0 / 255.0, 27.0 / 255.0, 27.0 / 255.0);

    const MINE_CHAR: &str = "☢";
    // const MINE_CHAR: &str = "X";
    const MINE_COLOR: Color = Self::COLOR_RED;
    const MINE_EXPLODED_CHAR: &str = "💥";
    // const MINE_EXPLODED_CHAR: &str = "#";
    const MINE_EXPLODED_COLOR: Color = Self::COLOR_RED;
    const FLAG_CHAR: &str = "⚐";
    // const FLAG_CHAR: &str = "f";
    const FLAG_COLOR_CORRECT: Color = Self::COLOR_GREEN;
    const FLAG_COLOR_WRONG: Color = Self::COLOR_RED;
    const EMPTY_SPOT_CHARS: [&str; 9] = [" ", "1", "2", "3", "4", "5", "6", "7", "8"];
    const EMPTY_SPOT_COLORS: [Color; Self::EMPTY_SPOT_CHARS.len()] = [
        Color::WHITE, Color::WHITE, Color::WHITE,
        Color::WHITE, Color::WHITE, Color::WHITE,
        Color::WHITE, Color::WHITE, Color::WHITE
    ];
    const REVEALED_SPOT_COLOR: Color = Self::COLOR_DARK_GRAY;
    const HIDDEN_SPOT_COLOR: Color = Self::COLOR_GRAY;

    const READY_COLOR: Color = Self::COLOR_GRAY;
    const WON_COLOR: Color = Self::COLOR_GREEN;
    const LOST_COLOR: Color = Self::COLOR_RED;
    
    const FLAG_COUNT_OK_COLOR: Color = Color::WHITE;
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
        let text_color = match self.game_state {
            GameState::Ready => Self::READY_COLOR,
            GameState::Running(_) => Color::WHITE,
            GameState::Paused(_) => Self::READY_COLOR,
            GameState::Stopped { is_won } => {
                match is_won {
                    true => Self::WON_COLOR,
                    false => Self::LOST_COLOR,
                }
            },
        };

        let time_text_size = 40;
        let time_text = match self.game_state {
            GameState::Ready => {
                widget::text("---").size(time_text_size)
            },
            GameState::Running(_) | GameState::Paused(_) => {
                widget::text(self.elapsed_seconds.as_secs()).size(time_text_size)
            },
            GameState::Stopped { is_won: _ } => {
                widget::text(self.elapsed_seconds.as_secs()).size(time_text_size)
            },
        };

        let display_seconds = widget::column![
            widget::text("Time").size(10).style(text_color),
            time_text.style(text_color)
        ]
        .align_items(Alignment::Center);
        

        let flags_text_size = 40;
    
        let flags_text = match self.game_state {
            GameState::Ready => {
                widget::text("---").size(flags_text_size).style(text_color)
            },
            GameState::Running(_) => {
                let flags_text_color = if self.remaining_flags >= 0 {
                    Self::FLAG_COUNT_OK_COLOR
                } else {
                    Self::FLAG_COUNT_ERR_COLOR
                };

                widget::text(self.remaining_flags).size(flags_text_size).style(flags_text_color)
            },
            GameState::Paused(_) => {
                widget::text(self.remaining_flags).size(flags_text_size).style(text_color)
            },
            GameState::Stopped { is_won: _} => {
                widget::text(self.remaining_flags).size(flags_text_size).style(text_color)
            },
        };
        let display_flags = widget::column![
            widget::text("Flags").size(10).style(text_color),
            flags_text
        ]
        .align_items(Alignment::Center);

        widget::row![
            widget::row![
                widget::button(widget::text(Self::REFRESH_BTN_CHAR).font(Self::COMMANDS_ICONS).size(20))
                    .on_press(Message::Reset)
                    .style(theme::Button::Primary),
            ]
             .width(Length::Shrink)
             .align_items(Alignment::Start),
            
            widget::row![
                widget::horizontal_space(Length::Fill),
                display_seconds,
                display_flags,
                widget::horizontal_space(Length::Fill)
            ]
             .spacing(20.0)
             .width(Length::Fill)
             .align_items(Alignment::Center),
            
            widget::row![
                widget::button(widget::text(Self::SETTINGS_BTN_CHAR).font(Self::MINES_FLAGS_ICONS))
                    .on_press(Message::Settings(SettingsMessage::Show))
                    .style(theme::Button::Primary),
                widget::button(widget::text(Self::ABOUT_BTN_CHAR).font(Self::COMMANDS_ICONS))
                    .on_press(Message::Info)
                    .style(theme::Button::Primary),
            ]
             .spacing(10.0)
             .width(Length::Shrink)
             .align_items(Alignment::End),
        ]
         .padding(10.0)
         .spacing(10.0)
         .align_items(Alignment::Center)
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

    fn view_settings(&self, game_difficulty: &GameDifficulty) -> Element<Message> {
        let mut settings_page = widget::column![
            widget::text("Game Difficulty"),
            widget::pick_list(GameDifficulty::ALL, Some(*game_difficulty), |x| { Message::Settings(SettingsMessage::Picked(x)) })    
        ].spacing(10.0);
            
        if let GameDifficulty::Custom(game_config) = game_difficulty {
            let width = game_config.width;
            let height = game_config.height;
            let mines = game_config.mines;

            let custom_game = widget::column![
                widget::text("Custom Game"),
                widget::row![
                    widget::text("Width:"),
                    widget::text_input("", game_config.width.to_string().as_str(), move |s| { 
                        if let Ok(i) = u16::from_str_radix(&s, 10) {
                            Message::Settings(SettingsMessage::ConfigWidth(i)) 
                        } else {
                            Message::Settings(SettingsMessage::ConfigWidth(width)) 
                        }
                    })
                ]
                 .spacing(10.0),
                widget::row![
                    widget::text("Height:"),
                    widget::text_input("", game_config.height.to_string().as_str(), move |s| { 
                        if let Ok(i) = u16::from_str_radix(&s, 10) {
                            Message::Settings(SettingsMessage::ConfigHeight(i)) 
                        } else {
                            Message::Settings(SettingsMessage::ConfigHeight(height)) 
                        }
                    })
                ]
                 .spacing(10.0),
                widget::row![
                    widget::text("Mines:"),
                    widget::text_input("", game_config.mines.to_string().as_str(), move |s| { 
                        if let Ok(i) = u32::from_str_radix(&s, 10) {
                            Message::Settings(SettingsMessage::ConfigMines(i)) 
                        } else {
                            Message::Settings(SettingsMessage::ConfigMines(mines)) 
                        }
                    })
                ]
                 .spacing(10.0),
            ].spacing(10.0);

            settings_page = settings_page.push(custom_game);
        }

        widget::column![
            settings_page
             .height(Length::Fill)
             .width(Length::Fill)
            ,
            widget::column![
                widget::row![
                    widget::button("Cancel")
                        .on_press(Message::Settings(SettingsMessage::Discard))
                        .style(theme::Button::Primary),
                    widget::button("Apply")
                        .on_press(Message::Settings(SettingsMessage::Set(*game_difficulty)))
                        .style(theme::Button::Primary),
                ]
                 .spacing(10.0)
                 .width(Length::Shrink)
                 .align_items(Alignment::End)
            ]
             .width(Length::Fill)
             .align_items(Alignment::End)


        ]
         .align_items(Alignment::End)
         .width(Length::Fill)
         .spacing(10.0)
         .padding(Self::FIELD_PAD)
         .into()
    }

    fn check_ready_to_running(&mut self) {
        if let GameState::Ready = self.game_state {
            self.elapsed_seconds = Duration::default();
            self.game_state = GameState::Running(Instant::now());
        }
    }
    
    fn game_over(&mut self, is_won: bool) {
        self.game_state = GameState::Stopped{is_won};
    }

    /// Pause the game, if it is running
    fn pause_game(&mut self) {
        if let GameState::Running(i) = self.game_state {
            let now = Instant::now();
            self.elapsed_seconds += now - i;
            self.game_state = GameState::Paused(now)
        }
    }

    /// Resume the game, if it is paused
    fn resume_game(&mut self) {
        if let GameState::Paused(i) = self.game_state {
            self.game_state = GameState::Running(Instant::now())
        }
    }

}

impl canvas::Program<Message> for Minesweep {
    type State = Interaction;

    fn update(
        &self,
        _interaction: &mut Interaction,
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
                        mouse::Event::ButtonPressed(mouse_button) => {
                            match mouse_button {
                                mouse::Button::Left => {
                                    (event::Status::Captured, Some(Message::Minesweep { message: MinesweepMessage::Step { x, y } }))
                                },
                                mouse::Button::Right => {
                                    (event::Status::Captured, Some(Message::Minesweep { message: MinesweepMessage::Flag { x, y } }))
                                },
                                mouse::Button::Middle => {
                                    (event::Status::Captured, Some(Message::Minesweep { message: MinesweepMessage::AutoStep { x, y } }))
                                },
                                mouse::Button::Other(_) => {
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
            let background_color = Self::REVEALED_SPOT_COLOR;
            frame.fill(&background, background_color);

            // determine where to draw the spots
            let f_width = self.field.width() as f32 * Self::SPOT_SIZE;
            let f_height = self.field.height() as f32 * Self::SPOT_SIZE;

            let f_o_x = (frame.width() - f_width) / 2.0;
            let f_o_y = (frame.height() - f_height) / 2.0;
            let origin_point = Point::new( f_o_x, f_o_y);

            // draw the spots
            for (&(ix, iy), spot) in self.field.spots() {
                let fx = (ix as f32 * Self::SPOT_SIZE) + Self::SPOT_PAD;
                let fy = (iy as f32 * Self::SPOT_SIZE) + Self::SPOT_PAD;
                let p = origin_point + Vector::new(fx, fy);
                
                let bounds = Rectangle::new(p, Size::new(Self::CELL_SIZE, Self::CELL_SIZE));
                let rounded_rectangle_radius = 0.0;

                let text = Text {
                    size: Self::CELL_SIZE,
                    position: bounds.center(),
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Center,
                    ..Text::default()
                };
                
                match spot.state {
                    minefield_rs::SpotState::HiddenEmpty { neighboring_mines: _ } => {
                        draw_rounded_rectangle(rounded_rectangle_radius, Self::HIDDEN_SPOT_COLOR, bounds, frame);
                    },
                    minefield_rs::SpotState::HiddenMine => {
                        draw_rounded_rectangle(rounded_rectangle_radius, Self::HIDDEN_SPOT_COLOR, bounds, frame);
                        
                        if let GameState::Stopped { is_won: _ } = self.game_state {
                            frame.fill_text(Text {
                                content: format!("{}", Self::MINE_CHAR),
                                position: text.position,
                                color: Self::MINE_COLOR,
                                font: Self::MINES_FLAGS_ICONS,
                                size: Self::CELL_SIZE - Self::CELL_PAD,
                                ..text
                            });
                        }
                    },
                    minefield_rs::SpotState::FlaggedEmpty { neighboring_mines: _ } => {
                        draw_rounded_rectangle(rounded_rectangle_radius, Self::HIDDEN_SPOT_COLOR, bounds, frame);
                        
                        let color = match self.game_state {
                            GameState::Ready | GameState::Running(_) | GameState::Paused(_) => {
                                Self::FLAG_COLOR_CORRECT
                            },
                            GameState::Stopped { is_won: _ } => {
                                Self::FLAG_COLOR_WRONG
                            },
                        };

                        frame.fill_text(Text {
                            content: format!("{}", Self::FLAG_CHAR),
                            position: text.position,
                            color,
                            font: Self::MINES_FLAGS_ICONS,
                            size: Self::CELL_SIZE - Self::CELL_PAD,
                            ..text
                        });
                    },
                    minefield_rs::SpotState::FlaggedMine => {
                        draw_rounded_rectangle(rounded_rectangle_radius, Self::HIDDEN_SPOT_COLOR, bounds, frame);

                        frame.fill_text(Text {
                            content: format!("{}", Self::FLAG_CHAR),
                            position: text.position,
                            color: Self::FLAG_COLOR_CORRECT,
                            font: Self::MINES_FLAGS_ICONS,
                            size: Self::CELL_SIZE - Self::CELL_PAD,
                            ..text
                        });
                    },
                    minefield_rs::SpotState::RevealedEmpty { neighboring_mines } => {
                        draw_rounded_rectangle(rounded_rectangle_radius, Self::REVEALED_SPOT_COLOR, bounds, frame);
                        
                        frame.fill_text(Text {
                            content: format!("{}", Self::EMPTY_SPOT_CHARS[neighboring_mines as usize]),
                            position: text.position,
                            color: Self::EMPTY_SPOT_COLORS[neighboring_mines as usize],
                            ..text
                        });
                    },
                    minefield_rs::SpotState::ExplodedMine => {
                        draw_rounded_rectangle(rounded_rectangle_radius, Self::REVEALED_SPOT_COLOR, bounds, frame);
                        
                        frame.fill_text(Text {
                            content: format!("{}", Self::MINE_EXPLODED_CHAR),
                            position: text.position,
                            color: Self::MINE_EXPLODED_COLOR,
                            font: Self::MINES_FLAGS_ICONS,
                            size: Self::CELL_SIZE - Self::CELL_PAD,
                            ..text
                        });
                    },
                }
            }
        });

        fn draw_rounded_rectangle(radius: f32, fill: Color, bounds: Rectangle, frame: &mut Frame) {
            let s_position = Point::new(bounds.position().x + (radius / 2.0), bounds.position().y);
            let s_size = Size::new(bounds.width - (radius * 1.0), bounds.height);

            frame.fill_rectangle(
                s_position,
                s_size,
                fill,
            );

            let wide_stroke = || -> Stroke {
                Stroke {
                    width: radius,
                    style: stroke::Style::Solid(fill),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };

            let left_line = Path::line(
                Point::new(bounds.position().x + (radius / 2.0), bounds.position().y + (radius / 2.0)), 
                Point::new(bounds.position().x + (radius / 2.0), bounds.position().y + bounds.height - (radius / 2.0))
            );
            frame.stroke(&left_line, wide_stroke());

            let right_line = Path::line(
                Point::new(bounds.position().x + (radius / 2.0) + s_size.width, bounds.position().y + (radius / 2.0)), 
                Point::new(bounds.position().x + (radius / 2.0) + s_size.width, bounds.position().y + bounds.height - (radius / 2.0))
            );
            frame.stroke(&right_line, wide_stroke());
        }

        vec![field]
    }
}

pub enum Interaction {
    None,
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
    Running (Instant),

    /// Game s paused at current time
    Paused(Instant),

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameDifficulty {
    Easy,
    Medium,
    Hard,
    Custom(GameConfig)
}

impl GameDifficulty {
    pub const ALL:&[GameDifficulty] = &[Self::Easy, Self::Medium, Self::Hard, Self::Custom(Self::DEFAULT_CUSTOM)];
    pub const EASY: GameConfig = GameConfig { width: 10, height: 10, mines: 10 };
    pub const MEDIUM: GameConfig = GameConfig { width: 16, height: 16, mines: 40 };
    pub const HARD: GameConfig = GameConfig { width: 30, height: 16, mines: 99 };
    pub const DEFAULT_CUSTOM: GameConfig = GameConfig{ width: 45, height: 24, mines: 150 };

    pub fn from_config(config: &GameConfig) -> Self {
        if *config == Self::EASY {
            Self::Easy
        } else if *config == Self::MEDIUM {
            Self::Medium
        } else if *config == Self::HARD {
            Self::Hard
        } else {
            Self::Custom(*config)
        }
    }

    pub fn to_config(&self) -> GameConfig {
        match self {
            GameDifficulty::Easy => Self::EASY,
            GameDifficulty::Medium => Self::MEDIUM,
            GameDifficulty::Hard => Self::HARD,
            GameDifficulty::Custom(gc) => *gc,
        }
    }
}

impl Display for GameDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameDifficulty::Easy => write!(f, "{} (w:{}, h:{}, m:{})", "Easy", Self::EASY.width, Self::EASY.height, Self::EASY.mines),
            GameDifficulty::Medium =>  write!(f, "{} (w:{}, h:{}, m:{})", "Medium", Self::MEDIUM.width, Self::MEDIUM.height, Self::MEDIUM.mines),
            GameDifficulty::Hard =>  write!(f, "{} (w:{}, h:{}, m:{})", "Hard", Self::HARD.width, Self::HARD.height, Self::HARD.mines),
            GameDifficulty::Custom(_) =>  write!(f, "{}", "Custom"),
        }
    }
}