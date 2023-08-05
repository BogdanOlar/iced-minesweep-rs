use iced::{
    alignment, executor,
    mouse::{self, Cursor},
    theme, time,
    widget::{
        self,
        canvas::{self, event, stroke, Cache, Event, Frame, LineCap, Path, Stroke, Text},
        container, Canvas,
    },
    Alignment, Application, Color, Command, Element, Font, Length, Point, Rectangle, Renderer,
    Size, Subscription, Theme, Vector,
};
use iced_runtime::font;
use minefield_rs::{FlagToggleResult, Minefield, StepResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::Display,
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub enum Message {
    /// Restart the game
    Reset,

    /// The info view has been requested
    Info,

    /// The high scores view has been requested
    HighScores,

    /// A new high score needs to be recorded
    HighScore(RecordHighScore),

    /// Messages related to game settings
    Settings(SettingsMessage),

    /// Messages related to playing the game
    Minesweep(MinesweepMessage),

    /// Load/Save game configs
    Persistance(PersistenceMessage),

    /// Message which informs us that a second has passed
    Tick(Instant),

    /// User provided fonts have been loaded
    FontLoaded(Result<(), font::Error>),
}

/// Lower level game logic messages
#[derive(Debug, Clone)]
pub enum MinesweepMessage {
    /// User is stepping on a spot
    Step { x: u16, y: u16 },

    /// User is autostepping around a spot
    AutoStep { x: u16, y: u16 },

    /// User is toggling a flag on a spot
    Flag { x: u16, y: u16 },
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    /// Show settings view
    Show,

    /// Apply the game configuration specified in the settings view
    Set(GameDifficulty),

    /// A new game difficulty has been picked, but not yet applied
    Picked(GameDifficulty),

    /// A new custom width has been entered, but not yet applied
    ConfigWidth(u16),

    /// A new custom height has been entered, but not yet applied
    ConfigHeight(u16),

    /// A new custom mine count has been entered, but not yet applied
    ConfigMines(u32),

    /// Discard the settings view without aplying any settings
    Discard,
}

#[derive(Debug, Clone)]
pub enum RecordHighScore {
    NameChanged(String),
    RecordName,
    Discard,
}

#[derive(Debug, Clone)]
pub enum PersistenceMessage {
    LoadedConfigs(Option<GamePersistence>),
    SavedConfigs,
}

#[derive(Debug, Clone)]
enum MainViewContent {
    /// Show the game (minefield) view
    Game,

    /// Show the settings view
    Settings(GameDifficulty),

    /// Show the Info view
    Info,

    /// Show the High Scores view
    HighScores,

    /// Show Record High Score view `Difficulty Level`, `seconds`, `name`
    EnterHighScore(DifficultyLevel, u64, String),
}

pub struct Minesweep {
    /// Model
    field: Minefield,

    /// View: a cache of the canvas holding the minefield. A redraw can be forced on it by calling `field_cache.clear()`
    field_cache: Cache,

    /// What the main view of the game is currently showing
    main_view: MainViewContent,

    /// Current state of the game
    game_state: GameState,

    /// Time duration since the beginning of game
    elapsed_seconds: Duration,

    /// Number of flags which still need to be placed by the player
    remaining_flags: i64,

    /// The specifications of the current game (width, height, number of mines)
    game_config: GameConfig,

    /// High Scores for each difficulty level
    high_scores: BTreeMap<DifficultyLevel, Vec<Score>>,
}

impl Application for Minesweep {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        let game_config = GameDifficulty::EASY;
        let high_scores = BTreeMap::new();

        let minesweep = Self {
            field: Minefield::new(game_config.width, game_config.height)
                .with_mines(game_config.mines),
            field_cache: Cache::default(),
            main_view: MainViewContent::Game,
            game_state: GameState::default(),
            game_config,
            elapsed_seconds: Duration::default(),
            remaining_flags: game_config.mines as i64,
            high_scores,
        };
        let (width, height) = minesweep.desired_window_size();

        let command = Command::batch(vec![
            iced_runtime::window::resize(Size { width, height }),
            Command::perform(Self::load_persistence(), |x| {
                Message::Persistance(PersistenceMessage::LoadedConfigs(x))
            }),
            iced::font::load(include_bytes!("../res/fonts/emoji-icon-font.ttf").as_slice())
                .map(Message::FontLoaded),
            iced::font::load(include_bytes!("../res/fonts/NotoEmoji-Regular.ttf").as_slice())
                .map(Message::FontLoaded),
            iced::font::load(include_bytes!("../res/fonts/Ubuntu-Light.ttf").as_slice())
                .map(Message::FontLoaded),
        ]);

        (minesweep, command)
    }

    fn title(&self) -> String {
        String::from(Self::APP_NAME)
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Minesweep(message) => {
                match message {
                    MinesweepMessage::Step { x, y } => {
                        self.check_ready_to_running();

                        if let GameState::Running(_) = self.game_state {
                            let step_result = self.field.step(x, y);

                            match step_result {
                                StepResult::Boom => {
                                    self.game_over(false);
                                }
                                StepResult::Phew => {
                                    if self.field.is_cleared() {
                                        self.game_over(true);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    MinesweepMessage::AutoStep { x, y } => {
                        if let GameState::Running(_) = self.game_state {
                            match self.field.auto_step(x, y) {
                                StepResult::Boom => {
                                    self.game_over(false);
                                }
                                StepResult::Phew => {
                                    if self.field.is_cleared() {
                                        self.game_over(true);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    MinesweepMessage::Flag { x, y } => {
                        self.check_ready_to_running();

                        if let GameState::Running(_) = self.game_state {
                            match self.field.toggle_flag(x, y) {
                                FlagToggleResult::Removed => {
                                    self.remaining_flags += 1;
                                }
                                FlagToggleResult::Added => {
                                    self.remaining_flags -= 1;

                                    if self.field.is_cleared() {
                                        self.game_over(true);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }

                self.field_cache.clear();
                Command::none()
            }
            Message::Reset => {
                self.field = Minefield::new(self.game_config.width, self.game_config.height)
                    .with_mines(self.game_config.mines);

                self.game_state = GameState::Ready;
                self.main_view = MainViewContent::Game;
                self.elapsed_seconds = Duration::default();
                self.remaining_flags = self.game_config.mines as i64;

                self.field_cache.clear();

                Command::none()
            }
            Message::Info => {
                match self.main_view {
                    MainViewContent::Info => {
                        // Get back to the game
                        self.resume_game();
                        self.main_view = MainViewContent::Game;
                    }
                    _ => {
                        self.pause_game();
                        self.main_view = MainViewContent::Info;
                    }
                }

                Command::none()
            }
            Message::Settings(settings_message) => {
                match settings_message {
                    SettingsMessage::Show => {
                        match self.main_view {
                            MainViewContent::Settings(_) => {
                                // Get back to the game
                                self.resume_game();
                                self.main_view = MainViewContent::Game;

                                Command::none()
                            }
                            _ => {
                                self.pause_game();
                                self.main_view = MainViewContent::Settings(
                                    GameDifficulty::from_config(&self.game_config),
                                );

                                Command::none()
                            }
                        }
                    }
                    SettingsMessage::Set(game_difficulty) => {
                        self.game_config = game_difficulty.into();

                        self.field =
                            Minefield::new(self.game_config.width, self.game_config.height)
                                .with_mines(self.game_config.mines);
                        self.game_state = GameState::Ready;
                        self.main_view = MainViewContent::Game;
                        self.elapsed_seconds = Duration::default();
                        self.remaining_flags = self.game_config.mines as i64;

                        let (width, height) = self.desired_window_size();

                        self.field_cache.clear();

                        let gp = GamePersistence {
                            game_config: self.game_config,
                            high_scores: self.high_scores.clone(),
                        };

                        Command::batch(vec![
                            iced_runtime::window::resize(Size { width, height }),
                            Command::perform(Self::save_persistence(gp), |_| {
                                Message::Persistance(PersistenceMessage::SavedConfigs)
                            }),
                        ])
                    }
                    SettingsMessage::Picked(gdif) => {
                        self.main_view = MainViewContent::Settings(gdif);

                        Command::none()
                    }
                    SettingsMessage::Discard => match self.main_view {
                        MainViewContent::Settings(_) => {
                            self.main_view = MainViewContent::Game;
                            self.resume_game();

                            Command::none()
                        }
                        _ => Command::none(),
                    },
                    SettingsMessage::ConfigWidth(width) => {
                        if let MainViewContent::Settings(GameDifficulty::Custom(game_config)) =
                            self.main_view
                        {
                            self.main_view =
                                MainViewContent::Settings(GameDifficulty::Custom(GameConfig {
                                    width,
                                    height: game_config.height,
                                    mines: game_config.mines,
                                }))
                        }
                        Command::none()
                    }
                    SettingsMessage::ConfigHeight(height) => {
                        if let MainViewContent::Settings(GameDifficulty::Custom(game_config)) =
                            self.main_view
                        {
                            self.main_view =
                                MainViewContent::Settings(GameDifficulty::Custom(GameConfig {
                                    width: game_config.width,
                                    height,
                                    mines: game_config.mines,
                                }))
                        }
                        Command::none()
                    }
                    SettingsMessage::ConfigMines(mines) => {
                        if let MainViewContent::Settings(GameDifficulty::Custom(game_config)) =
                            self.main_view
                        {
                            self.main_view =
                                MainViewContent::Settings(GameDifficulty::Custom(GameConfig {
                                    width: game_config.width,
                                    height: game_config.height,
                                    mines,
                                }))
                        }
                        Command::none()
                    }
                }
            }

            Message::HighScores => {
                match self.main_view {
                    MainViewContent::HighScores => {
                        // Get back to the game
                        self.resume_game();
                        self.main_view = MainViewContent::Game;
                    }
                    _ => {
                        self.pause_game();
                        self.main_view = MainViewContent::HighScores;
                    }
                }

                Command::none()
            }

            Message::Tick(new_tick) => {
                if let GameState::Running(cur_tick) = &mut self.game_state {
                    self.elapsed_seconds += new_tick - *cur_tick;
                    *cur_tick = new_tick;
                }

                Command::none()
            }
            Message::HighScore(rec) => {
                match rec {
                    RecordHighScore::NameChanged(name) => {
                        if let MainViewContent::EnterHighScore(d, s, old_name) =
                            self.main_view.clone()
                        {
                            // Enforce maximum name length
                            if name.chars().count() < Self::MAX_HIGHSCORE_NAME_LEN {
                                self.main_view = MainViewContent::EnterHighScore(d, s, name);
                            } else {
                                self.main_view = MainViewContent::EnterHighScore(d, s, old_name);
                            }
                        }

                        Command::none()
                    }
                    RecordHighScore::RecordName => {
                        if let MainViewContent::EnterHighScore(difficulty_level, seconds, name) =
                            self.main_view.clone()
                        {
                            self.main_view = MainViewContent::HighScores;
                            self.insert_high_score(difficulty_level, seconds, name);

                            let gp = GamePersistence {
                                game_config: self.game_config,
                                high_scores: self.high_scores.clone(),
                            };

                            Command::perform(Self::save_persistence(gp), |_| {
                                Message::Persistance(PersistenceMessage::SavedConfigs)
                            })
                        } else {
                            Command::none()
                        }
                    }
                    RecordHighScore::Discard => {
                        if let MainViewContent::EnterHighScore(_, _, _) = self.main_view {
                            self.main_view = MainViewContent::Game;
                        }

                        Command::none()
                    }
                }
            }
            Message::Persistance(pmsg) => {
                let command;

                match pmsg {
                    PersistenceMessage::LoadedConfigs(game_p) => {
                        if let Some(game_p) = game_p {
                            // load High Scores
                            self.high_scores = game_p.high_scores;

                            // Load game config, if it's not custom
                            let game_difficulty = GameDifficulty::from_config(&game_p.game_config);

                            match game_difficulty {
                                GameDifficulty::Easy
                                | GameDifficulty::Medium
                                | GameDifficulty::Hard => {
                                    // Apply the game config loaded from file
                                    command = Command::perform(
                                        async move {
                                            Message::Settings(SettingsMessage::Set(game_difficulty))
                                        },
                                        |m| m,
                                    )
                                }
                                GameDifficulty::Custom(_) => {
                                    // FIXME: wrong custom configs can crash the game or make it unusable
                                    command = Command::none();
                                }
                            }
                        } else {
                            command = Command::none();
                        }
                    }
                    PersistenceMessage::SavedConfigs => {
                        command = Command::none();
                    }
                }

                command
            }
            Message::FontLoaded(_) => Command::none(),
        }
    }

    fn view(&self) -> iced::Element<'_, Self::Message, iced::Renderer<Self::Theme>> {
        let main_view = match &self.main_view {
            MainViewContent::Game => self.view_field(),
            MainViewContent::Settings(game_difficulty) => self.view_settings(game_difficulty),
            MainViewContent::Info => self.view_info(),
            MainViewContent::HighScores => {
                // self.view_high_scores().explain(Color::WHITE)
                self.view_high_scores()
            }
            MainViewContent::EnterHighScore(difficulty_level, seconds, name) => {
                self.view_record_high_score(*difficulty_level, *seconds, name)
            }
        };

        let content = widget::column![self.view_controls(), main_view]
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
    const MINES_FLAGS_ICONS: Font = Font::with_name("emoji");

    // Fonts for mines and flags
    const COMMANDS_ICONS: Font = Font::with_name("Noto Emoji");

    // Fonts for text
    const TEXT_FONT: Font = Font::with_name("Ubuntu Light");

    const LICESE_BYTES: &'static [u8] = include_bytes!("../LICENSE");

    const REFRESH_BTN_CHAR: &str = "🔄";
    const SETTINGS_BTN_CHAR: &str = "🛠";
    const ABOUT_BTN_CHAR: &str = "ℹ";
    const HIGH_SCORES_CHAR: &str = "🏆";

    const TOOLBAR_HEIGHT: f32 = 70.0;
    const FIELD_PAD: f32 = 20.0;
    /// Size of spor on canvas, including padding
    const SPOT_SIZE: f32 = 30.0;
    /// Interior padding of spot
    const SPOT_PAD: f32 = 1.0;
    const CELL_SIZE: f32 = Self::SPOT_SIZE - (Self::SPOT_PAD * 2.0);
    const CELL_PAD: f32 = 8.0;

    #[allow(clippy::eq_op)]
    const COLOR_RED: Color = Color::from_rgb(255.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0);
    #[allow(clippy::eq_op)]
    const COLOR_LIGHT_RED: Color = Color::from_rgb(255.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0);
    #[allow(clippy::eq_op)]
    const COLOR_GREEN: Color = Color::from_rgb(0.0 / 255.0, 255.0 / 255.0, 0.0 / 255.0);
    const COLOR_GRAY: Color = Color::from_rgb(60.0 / 255.0, 60.0 / 255.0, 60.0 / 255.0);
    const COLOR_DARK_GRAY: Color = Color::from_rgb(27.0 / 255.0, 27.0 / 255.0, 27.0 / 255.0);

    const MINE_CHAR: &str = "☢";
    const MINE_COLOR: Color = Self::COLOR_RED;
    const MINE_EXPLODED_CHAR: &str = "💥";
    const MINE_EXPLODED_COLOR: Color = Self::COLOR_RED;
    const FLAG_CHAR: &str = "⚐";
    const FLAG_COLOR_CORRECT: Color = Self::COLOR_GREEN;
    const FLAG_COLOR_WRONG: Color = Self::COLOR_RED;
    const EMPTY_SPOT_CHARS: [&str; 9] = [" ", "1", "2", "3", "4", "5", "6", "7", "8"];
    const EMPTY_SPOT_COLORS: [Color; Self::EMPTY_SPOT_CHARS.len()] = [
        Color::WHITE,
        Color::WHITE,
        Color::WHITE,
        Color::WHITE,
        Color::WHITE,
        Color::WHITE,
        Color::WHITE,
        Color::WHITE,
        Color::WHITE,
    ];
    const REVEALED_SPOT_COLOR: Color = Self::COLOR_DARK_GRAY;
    const HIDDEN_SPOT_COLOR: Color = Self::COLOR_GRAY;

    const READY_COLOR: Color = Self::COLOR_GRAY;
    const WON_COLOR: Color = Self::COLOR_GREEN;
    const LOST_COLOR: Color = Self::COLOR_RED;

    const FLAG_COUNT_OK_COLOR: Color = Color::WHITE;
    const FLAG_COUNT_ERR_COLOR: Color = Self::COLOR_LIGHT_RED;

    const MAX_HIGH_SCORES_PER_LEVEL: usize = 3;
    const DEFAULT_NAME: &str = "Anonymous";
    const MAX_HIGHSCORE_NAME_LEN: usize = 32;

    #[allow(dead_code)]
    pub fn with_configs(mut self, game_config: GameConfig) -> Self {
        self.game_config = game_config;
        self.field = Minefield::new(self.game_config.width, self.game_config.height)
            .with_mines(self.game_config.mines);

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

    /// Controls view
    fn view_controls(&self) -> Element<Message> {
        let text_color = match self.game_state {
            GameState::Ready => Self::READY_COLOR,
            GameState::Running(_) => Color::WHITE,
            GameState::Paused => Self::READY_COLOR,
            GameState::Stopped { is_won } => match is_won {
                true => Self::WON_COLOR,
                false => Self::LOST_COLOR,
            },
        };

        let time_text_size = 40;
        let time_text = match self.game_state {
            GameState::Ready => widget::text("---").size(time_text_size),
            GameState::Running(_) | GameState::Paused => {
                widget::text(self.elapsed_seconds.as_secs()).size(time_text_size)
            }
            GameState::Stopped { is_won: _ } => {
                widget::text(self.elapsed_seconds.as_secs()).size(time_text_size)
            }
        };

        let display_seconds = widget::column![
            widget::text("Time").size(10).style(text_color),
            time_text.style(text_color)
        ]
        .align_items(Alignment::Center);

        let flags_text_size = 40;

        let flags_text = match self.game_state {
            GameState::Ready => widget::text("---").size(flags_text_size).style(text_color),
            GameState::Running(_) => {
                let flags_text_color = if self.remaining_flags >= 0 {
                    Self::FLAG_COUNT_OK_COLOR
                } else {
                    Self::FLAG_COUNT_ERR_COLOR
                };

                widget::text(self.remaining_flags)
                    .size(flags_text_size)
                    .style(flags_text_color)
            }
            GameState::Paused => widget::text(self.remaining_flags)
                .size(flags_text_size)
                .style(text_color),
            GameState::Stopped { is_won: _ } => widget::text(self.remaining_flags)
                .size(flags_text_size)
                .style(text_color),
        };
        let display_flags =
            widget::column![widget::text("Flags").size(10).style(text_color), flags_text]
                .align_items(Alignment::Center);

        widget::row![
            widget::row![widget::button(
                widget::text(Self::REFRESH_BTN_CHAR)
                    .font(Self::COMMANDS_ICONS)
                    .size(20)
            )
            .on_press(Message::Reset)
            .style(theme::Button::Primary),]
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
                widget::button(widget::text(Self::HIGH_SCORES_CHAR).font(Self::COMMANDS_ICONS))
                    .on_press(Message::HighScores)
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

    /// Minefield view
    fn view_field(&self) -> Element<Message> {
        let (field_width, field_height) = self.desired_field_size();
        Canvas::new(self)
            .width(field_width)
            .height(field_height)
            .into()
    }

    /// Settings view
    fn view_settings(&self, game_difficulty: &GameDifficulty) -> Element<Message> {
        let mut settings_page = widget::column![
            widget::text("Game Difficulty"),
            widget::pick_list(GameDifficulty::ALL, Some(*game_difficulty), |x| {
                Message::Settings(SettingsMessage::Picked(x))
            })
        ]
        .spacing(10.0);

        if let GameDifficulty::Custom(game_config) = game_difficulty {
            let width = game_config.width;
            let height = game_config.height;
            let mines = game_config.mines;

            let custom_game = widget::column![
                widget::text("Custom Game"),
                widget::row![
                    widget::text("Width:"),
                    widget::text_input("", game_config.width.to_string().as_str()).on_input(
                        move |s| {
                            if let Ok(i) = s.parse::<u16>() {
                                Message::Settings(SettingsMessage::ConfigWidth(i))
                            } else {
                                Message::Settings(SettingsMessage::ConfigWidth(width))
                            }
                        }
                    )
                ]
                .spacing(10.0),
                widget::row![
                    widget::text("Height:"),
                    widget::text_input("", game_config.height.to_string().as_str()).on_input(
                        move |s| {
                            if let Ok(i) = s.parse::<u16>() {
                                Message::Settings(SettingsMessage::ConfigHeight(i))
                            } else {
                                Message::Settings(SettingsMessage::ConfigHeight(height))
                            }
                        }
                    )
                ]
                .spacing(10.0),
                widget::row![
                    widget::text("Mines:"),
                    widget::text_input("", game_config.mines.to_string().as_str()).on_input(
                        move |s| {
                            if let Ok(i) = s.parse::<u32>() {
                                Message::Settings(SettingsMessage::ConfigMines(i))
                            } else {
                                Message::Settings(SettingsMessage::ConfigMines(mines))
                            }
                        }
                    )
                ]
                .spacing(10.0),
            ]
            .spacing(10.0);

            settings_page = settings_page.push(custom_game);
        }

        widget::column![
            settings_page.height(Length::Fill).width(Length::Fill),
            widget::column![widget::row![
                widget::button("Cancel")
                    .on_press(Message::Settings(SettingsMessage::Discard))
                    .style(theme::Button::Primary),
                widget::button("Apply")
                    .on_press(Message::Settings(SettingsMessage::Set(*game_difficulty)))
                    .style(theme::Button::Primary),
            ]
            .spacing(10.0)
            .width(Length::Shrink)
            .align_items(Alignment::End)]
            .width(Length::Fill)
            .align_items(Alignment::End)
        ]
        .align_items(Alignment::End)
        .width(Length::Fill)
        .spacing(10.0)
        .padding(Self::FIELD_PAD)
        .into()
    }

    /// Info/"About" view
    fn view_info(&self) -> Element<Message> {
        let license_text = std::str::from_utf8(Self::LICESE_BYTES).unwrap_or("");

        let content = widget::column![
            widget::row![widget::text("About").font(Self::TEXT_FONT)],
            widget::row![widget::text("Copyright (c) 2023 Bogdan Olar").size(15.0)].padding(10),
            widget::row![
                widget::text("https://github.com/BogdanOlar/iced-minesweep-rs").size(15.0)
            ]
            .padding(10),
            widget::row![widget::text("License").font(Self::TEXT_FONT)],
            widget::row![widget::text(license_text).font(Self::TEXT_FONT).size(12.0)].padding(10),
            widget::column![widget::row![widget::button("Ok")
                .on_press(Message::Info)
                .style(theme::Button::Primary),]
            .spacing(10.0)
            .width(Length::Shrink)
            .align_items(Alignment::End)]
            .width(Length::Fill)
            .align_items(Alignment::End)
            .padding(20.0)
        ]
        .align_items(Alignment::Start)
        .spacing(10);

        widget::column![widget::scrollable(container(content).width(Length::Fill)),]
            .padding(Self::FIELD_PAD)
            .into()
    }

    /// High Scores view
    fn view_high_scores(&self) -> Element<Message> {
        let mut content = widget::column![]
            .spacing(10)
            .width(Length::Fill)
            .padding(20.0);
        content = content.push(
            widget::column![widget::text("High Scores").font(Self::TEXT_FONT).size(25.0)]
                .width(Length::Fill)
                .align_items(Alignment::Center),
        );

        for difficulty_level in DifficultyLevel::ALL {
            content = content.push(widget::horizontal_rule(10.0));

            content = content.push(
                widget::row![widget::text(difficulty_level.to_string()).font(Self::TEXT_FONT)]
                    .width(Length::Fill)
                    .align_items(Alignment::Center),
            );

            let empty_scores = Vec::new();

            let scores = if let Some(scores) = self.high_scores.get(difficulty_level) {
                scores
            } else {
                &empty_scores
            };

            for i in 0..Self::MAX_HIGH_SCORES_PER_LEVEL {
                if let Some(score) = scores.get(i) {
                    content = content.push(
                        widget::row![
                            widget::column![widget::text(format!("# {}. ", i + 1)).size(15.0),]
                                .width(Length::Shrink)
                                .height(Length::Shrink)
                                .align_items(Alignment::Start),
                            widget::column![widget::text(score.name.as_str()).size(15.0)]
                                .width(Length::Fill)
                                .height(Length::Shrink)
                                .align_items(Alignment::Start),
                            widget::column![widget::text(score.seconds.to_string()).size(15.0)]
                                .width(Length::Shrink)
                                .height(Length::Shrink)
                                .align_items(Alignment::End),
                            widget::horizontal_space(Length::Fill),
                        ]
                        .width(Length::Fill)
                        .spacing(40.0)
                        .align_items(Alignment::End),
                    );
                } else {
                    content = content.push(
                        widget::row![
                            widget::column![widget::text(format!("# {}. ", i + 1))
                                .size(15.0)
                                .style(Self::READY_COLOR),]
                            .width(Length::Shrink)
                            .height(Length::Shrink)
                            .align_items(Alignment::Start),
                            widget::column![widget::text("Empty")
                                .size(15.0)
                                .style(Self::READY_COLOR),]
                            .width(Length::Fill)
                            .height(Length::Shrink)
                            .align_items(Alignment::Start),
                            widget::horizontal_space(Length::Fill),
                        ]
                        .width(Length::Fill)
                        .spacing(40.0)
                        .align_items(Alignment::End),
                    );
                }
            }
        }

        content = content.push(
            widget::column![widget::row![widget::button("Ok")
                .on_press(Message::HighScores)
                .style(theme::Button::Primary),]
            .spacing(10.0)
            .width(Length::Shrink)
            .align_items(Alignment::End)]
            .width(Length::Fill)
            .align_items(Alignment::End)
            .padding(20.0),
        );

        widget::column![widget::scrollable(container(content).width(Length::Fill)),]
            .width(Length::Fill)
            .padding(Self::FIELD_PAD)
            .into()
    }

    fn view_record_high_score(
        &self,
        difficulty_level: DifficultyLevel,
        _: u64,
        name: &str,
    ) -> Element<Message> {
        let record_hs_page = widget::column![
            widget::column![
                widget::text("New HIGH SCORE!")
                    .font(Self::TEXT_FONT)
                    .size(25.0),
                widget::text(format!("({} difficulty level)", difficulty_level))
                    .font(Self::TEXT_FONT)
                    .size(10.0),
            ]
            .width(Length::Fill)
            .align_items(Alignment::Center),
            widget::row![widget::text_input("Please enter your name", name)
                .on_input(move |s| { Message::HighScore(RecordHighScore::NameChanged(s)) })
                .on_submit(Message::HighScore(RecordHighScore::RecordName))]
            .spacing(10.0)
        ]
        .spacing(10.0)
        .width(Length::Fill)
        .padding(20.0);

        widget::column![
            record_hs_page.height(Length::Fill).width(Length::Fill),
            widget::column![widget::row![
                widget::button("Cancel")
                    .on_press(Message::HighScore(RecordHighScore::Discard))
                    .style(theme::Button::Primary),
                widget::button("Apply")
                    .on_press(Message::HighScore(RecordHighScore::RecordName))
                    .style(theme::Button::Primary),
            ]
            .spacing(10.0)
            .width(Length::Shrink)
            .align_items(Alignment::End)]
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
        self.game_state = GameState::Stopped { is_won };

        if is_won {
            let seconds = self.elapsed_seconds.as_secs();

            match GameDifficulty::from_config(&self.game_config) {
                GameDifficulty::Easy => {
                    if self.is_high_score(DifficultyLevel::Easy, seconds) {
                        self.main_view = MainViewContent::EnterHighScore(
                            DifficultyLevel::Easy,
                            seconds,
                            String::new(),
                        );
                    }
                }
                GameDifficulty::Medium => {
                    if self.is_high_score(DifficultyLevel::Medium, seconds) {
                        self.main_view = MainViewContent::EnterHighScore(
                            DifficultyLevel::Medium,
                            seconds,
                            String::new(),
                        );
                    }
                }
                GameDifficulty::Hard => {
                    if self.is_high_score(DifficultyLevel::Hard, seconds) {
                        self.main_view = MainViewContent::EnterHighScore(
                            DifficultyLevel::Hard,
                            seconds,
                            String::new(),
                        );
                    }
                }
                GameDifficulty::Custom(_) => {}
            }
        }
    }

    fn is_high_score(&self, level: DifficultyLevel, seconds: u64) -> bool {
        if let Some(scores) = self.high_scores.get(&level) {
            if scores.len() >= Self::MAX_HIGH_SCORES_PER_LEVEL {
                let max = scores
                    .iter()
                    .max_by(|&s1, &s2| s1.seconds.cmp(&s2.seconds))
                    .map(|s| s.seconds)
                    .unwrap();

                // compare highest time with current time
                seconds < max
            } else {
                // high scores list for this level is not full
                true
            }
        } else {
            // no recorded high scores for this level
            true
        }
    }

    fn insert_high_score(&mut self, difficulty_level: DifficultyLevel, seconds: u64, name: String) {
        let name = if name.is_empty() {
            Self::DEFAULT_NAME.to_string()
        } else {
            name
        };
        if let Some(scores) = self.high_scores.get_mut(&difficulty_level) {
            scores.push(Score { name, seconds });

            scores.sort_by(|s1, s2| s1.seconds.cmp(&s2.seconds));

            while scores.len() > Self::MAX_HIGH_SCORES_PER_LEVEL {
                scores.pop();
            }
        } else {
            self.high_scores
                .insert(difficulty_level, vec![Score { name, seconds }]);
        }
    }

    /// Pause the game, if it is running
    fn pause_game(&mut self) {
        if let GameState::Running(i) = self.game_state {
            let now = Instant::now();
            self.elapsed_seconds += now - i;
            self.game_state = GameState::Paused
        }
    }

    /// Resume the game, if it is paused
    fn resume_game(&mut self) {
        if let GameState::Paused = self.game_state {
            self.game_state = GameState::Running(Instant::now())
        }
    }

    pub async fn load_persistence() -> Option<GamePersistence> {
        let path = Self::APP_NAME.to_owned() + ".json";
        if let Ok(mut file) = std::fs::File::open(path) {
            let mut buf = vec![];
            if std::io::Read::read_to_end(&mut file, &mut buf).is_ok() {
                if let Ok(world) = serde_json::from_slice(&buf[..]) {
                    return Some(world);
                }
            }
        }

        None
    }

    pub async fn save_persistence(configs: GamePersistence) {
        let path = Self::APP_NAME.to_owned() + ".json";
        if let Ok(mut f) = std::fs::File::create(path) {
            if let Ok(buf) = serde_json::to_vec(&configs) {
                let _ = std::io::Write::write_all(&mut f, &buf[..]);
            }
        }
    }
}

impl canvas::Program<Message> for Minesweep {
    type State = ();

    fn update(
        &self,
        _interaction: &mut Self::State,
        event: Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (event::Status, Option<Message>) {
        // determine where to draw the spots
        let f_width = self.field.width() as f32 * Self::SPOT_SIZE;
        let f_height = self.field.height() as f32 * Self::SPOT_SIZE;

        let f_o_x = (bounds.width - f_width) / 2.0;
        let f_o_y = (bounds.height - f_height) / 2.0;
        let origin_point = Point::new(bounds.x + f_o_x, bounds.y + f_o_y);
        let origin_rectangle = Rectangle::new(origin_point, Size::new(f_width, f_height));

        if let Some(position) = cursor.position_in(origin_rectangle) {
            let x = (position.x / Self::SPOT_SIZE).floor() as u16;
            let y = (position.y / Self::SPOT_SIZE).floor() as u16;

            match event {
                Event::Mouse(mouse_event) => match mouse_event {
                    mouse::Event::ButtonPressed(mouse_button) => match mouse_button {
                        mouse::Button::Left => (
                            event::Status::Captured,
                            Some(Message::Minesweep(MinesweepMessage::Step { x, y })),
                        ),
                        mouse::Button::Right => (
                            event::Status::Captured,
                            Some(Message::Minesweep(MinesweepMessage::Flag { x, y })),
                        ),
                        mouse::Button::Middle => (
                            event::Status::Captured,
                            Some(Message::Minesweep(MinesweepMessage::AutoStep { x, y })),
                        ),
                        mouse::Button::Other(_) => (event::Status::Ignored, None),
                    },
                    _ => (event::Status::Ignored, None),
                },
                Event::Touch(_t) => {
                    // TODO: add handling for touch (WASM on mobile devices)
                    (event::Status::Ignored, None)
                }
                Event::Keyboard(_) => (event::Status::Ignored, None),
            }
        } else {
            (event::Status::Ignored, None)
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        let field = self.field_cache.draw(renderer, bounds.size(), |frame| {
            // Set the background
            let background = Path::rectangle(Point::ORIGIN, frame.size());
            let background_color = Self::REVEALED_SPOT_COLOR;
            frame.fill(&background, background_color);

            // determine where to draw the spots
            let f_width = self.field.width() as f32 * Self::SPOT_SIZE;
            let f_height = self.field.height() as f32 * Self::SPOT_SIZE;

            let f_o_x = (frame.width() - f_width) / 2.0;
            let f_o_y = (frame.height() - f_height) / 2.0;
            let origin_point = Point::new(f_o_x, f_o_y);

            // draw the spots
            for (&(ix, iy), spot) in self.field.spots() {
                let fx = (ix as f32 * Self::SPOT_SIZE) + Self::SPOT_PAD;
                let fy = (iy as f32 * Self::SPOT_SIZE) + Self::SPOT_PAD;
                let p = origin_point + Vector::new(fx, fy);

                let bounds = Rectangle::new(p, Size::new(Self::CELL_SIZE, Self::CELL_SIZE));
                let rounded_rectangle_radius = 0.0;

                let text = Text {
                    size: Self::CELL_SIZE - Self::CELL_PAD,
                    position: bounds.center(),
                    horizontal_alignment: alignment::Horizontal::Center,
                    vertical_alignment: alignment::Vertical::Center,
                    ..Text::default()
                };

                match spot.state {
                    minefield_rs::SpotState::HiddenEmpty {
                        neighboring_mines: _,
                    } => {
                        draw_rounded_rectangle(
                            rounded_rectangle_radius,
                            Self::HIDDEN_SPOT_COLOR,
                            bounds,
                            frame,
                        );
                    }
                    minefield_rs::SpotState::HiddenMine => {
                        draw_rounded_rectangle(
                            rounded_rectangle_radius,
                            Self::HIDDEN_SPOT_COLOR,
                            bounds,
                            frame,
                        );

                        if let GameState::Stopped { is_won: _ } = self.game_state {
                            frame.fill_text(Text {
                                content: Self::MINE_CHAR.to_string(),
                                position: text.position,
                                color: Self::MINE_COLOR,
                                font: Self::MINES_FLAGS_ICONS,
                                size: Self::CELL_SIZE - Self::CELL_PAD,
                                ..text
                            });
                        }
                    }
                    minefield_rs::SpotState::FlaggedEmpty {
                        neighboring_mines: _,
                    } => {
                        draw_rounded_rectangle(
                            rounded_rectangle_radius,
                            Self::HIDDEN_SPOT_COLOR,
                            bounds,
                            frame,
                        );

                        let color = match self.game_state {
                            GameState::Ready | GameState::Running(_) | GameState::Paused => {
                                Self::FLAG_COLOR_CORRECT
                            }
                            GameState::Stopped { is_won: _ } => Self::FLAG_COLOR_WRONG,
                        };

                        frame.fill_text(Text {
                            content: Self::FLAG_CHAR.to_string(),
                            position: text.position,
                            color,
                            font: Self::MINES_FLAGS_ICONS,
                            size: Self::CELL_SIZE - Self::CELL_PAD,
                            ..text
                        });
                    }
                    minefield_rs::SpotState::FlaggedMine => {
                        draw_rounded_rectangle(
                            rounded_rectangle_radius,
                            Self::HIDDEN_SPOT_COLOR,
                            bounds,
                            frame,
                        );

                        frame.fill_text(Text {
                            content: Self::FLAG_CHAR.to_string(),
                            position: text.position,
                            color: Self::FLAG_COLOR_CORRECT,
                            font: Self::MINES_FLAGS_ICONS,
                            size: Self::CELL_SIZE - Self::CELL_PAD,
                            ..text
                        });
                    }
                    minefield_rs::SpotState::RevealedEmpty { neighboring_mines } => {
                        draw_rounded_rectangle(
                            rounded_rectangle_radius,
                            Self::REVEALED_SPOT_COLOR,
                            bounds,
                            frame,
                        );

                        frame.fill_text(Text {
                            content: Self::EMPTY_SPOT_CHARS[neighboring_mines as usize].to_string(),
                            position: text.position,
                            color: Self::EMPTY_SPOT_COLORS[neighboring_mines as usize],
                            ..text
                        });
                    }
                    minefield_rs::SpotState::ExplodedMine => {
                        draw_rounded_rectangle(
                            rounded_rectangle_radius,
                            Self::REVEALED_SPOT_COLOR,
                            bounds,
                            frame,
                        );

                        frame.fill_text(Text {
                            content: Self::MINE_EXPLODED_CHAR.to_string(),
                            position: text.position,
                            color: Self::MINE_EXPLODED_COLOR,
                            font: Self::MINES_FLAGS_ICONS,
                            size: Self::CELL_SIZE - Self::CELL_PAD,
                            ..text
                        });
                    }
                }
            }
        });

        fn draw_rounded_rectangle(radius: f32, fill: Color, bounds: Rectangle, frame: &mut Frame) {
            let s_position = Point::new(bounds.position().x + (radius / 2.0), bounds.position().y);
            let s_size = Size::new(bounds.width - (radius * 1.0), bounds.height);

            frame.fill_rectangle(s_position, s_size, fill);

            let wide_stroke = || -> Stroke {
                Stroke {
                    width: radius,
                    style: stroke::Style::Solid(fill),
                    line_cap: LineCap::Round,
                    ..Stroke::default()
                }
            };

            let left_line = Path::line(
                Point::new(
                    bounds.position().x + (radius / 2.0),
                    bounds.position().y + (radius / 2.0),
                ),
                Point::new(
                    bounds.position().x + (radius / 2.0),
                    bounds.position().y + bounds.height - (radius / 2.0),
                ),
            );
            frame.stroke(&left_line, wide_stroke());

            let right_line = Path::line(
                Point::new(
                    bounds.position().x + (radius / 2.0) + s_size.width,
                    bounds.position().y + (radius / 2.0),
                ),
                Point::new(
                    bounds.position().x + (radius / 2.0) + s_size.width,
                    bounds.position().y + bounds.height - (radius / 2.0),
                ),
            );
            frame.stroke(&right_line, wide_stroke());
        }

        vec![field]
    }
}

/// Current state of the game
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum GameState {
    /// Game is ready to start running
    Ready,

    /// Game is running
    Running(Instant),

    /// Game is paused
    Paused,

    /// Game is stopped, and was either won (`true`), or lost (`false`)
    Stopped { is_won: bool },
}

impl Default for GameState {
    fn default() -> Self {
        Self::Ready
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameConfig {
    pub width: u16,
    pub height: u16,
    pub mines: u32,
}

impl From<GameDifficulty> for GameConfig {
    fn from(val: GameDifficulty) -> Self {
        match val {
            GameDifficulty::Easy => GameDifficulty::EASY,
            GameDifficulty::Medium => GameDifficulty::MEDIUM,
            GameDifficulty::Hard => GameDifficulty::HARD,
            GameDifficulty::Custom(gc) => gc,
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            width: 10,
            height: 10,
            mines: 10,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameDifficulty {
    Easy,
    Medium,
    Hard,
    Custom(GameConfig),
}

impl GameDifficulty {
    pub const ALL: &[GameDifficulty] = &[
        Self::Easy,
        Self::Medium,
        Self::Hard,
        Self::Custom(Self::DEFAULT_CUSTOM),
    ];
    pub const EASY: GameConfig = GameConfig {
        width: 10,
        height: 10,
        mines: 10,
    };
    pub const MEDIUM: GameConfig = GameConfig {
        width: 16,
        height: 16,
        mines: 40,
    };
    pub const HARD: GameConfig = GameConfig {
        width: 30,
        height: 16,
        mines: 99,
    };
    pub const DEFAULT_CUSTOM: GameConfig = GameConfig {
        width: 45,
        height: 24,
        mines: 150,
    };

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
}

impl Display for GameDifficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameDifficulty::Easy => write!(
                f,
                "Easy (w:{}, h:{}, m:{})",
                Self::EASY.width,
                Self::EASY.height,
                Self::EASY.mines
            ),
            GameDifficulty::Medium => write!(
                f,
                "Medium (w:{}, h:{}, m:{})",
                Self::MEDIUM.width,
                Self::MEDIUM.height,
                Self::MEDIUM.mines
            ),
            GameDifficulty::Hard => write!(
                f,
                "Hard (w:{}, h:{}, m:{})",
                Self::HARD.width,
                Self::HARD.height,
                Self::HARD.mines
            ),
            GameDifficulty::Custom(gc) => write!(
                f,
                "Custom (w:{}, h:{}, m:{})",
                gc.width, gc.height, gc.mines
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Easy,
    Medium,
    Hard,
}

impl DifficultyLevel {
    pub const ALL: &[DifficultyLevel] = &[Self::Easy, Self::Medium, Self::Hard];
}

impl Display for DifficultyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DifficultyLevel::Easy => write!(f, "Easy"),
            DifficultyLevel::Medium => write!(f, "Medium"),
            DifficultyLevel::Hard => write!(f, "Hard"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    name: String,
    seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GamePersistence {
    game_config: GameConfig,
    high_scores: BTreeMap<DifficultyLevel, Vec<Score>>,
}
