use std::io;
use std::sync::mpsc;
use std::{thread, time::Duration};

use clap::{Parser, ValueEnum};
use game_of_life::game::cell::Cell;
use game_of_life::game::{CellBoardCreationError, FormatErrorVariant};

use crate::commands::CliCommand;
use crossterm::event::{KeyCode, KeyEventKind};
use game_of_life::game::{Game, cell::Slot};
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::{crossterm, prelude::*};

#[derive(Clone, Copy, ValueEnum)]
enum SpeedVariant {
    Slow,
    Normal,
    Fast,
}

#[derive(Clone, Copy)]
enum Speed {
    Paused,
    Unpaused(SpeedVariant),
}

impl Speed {
    pub fn regulate(&mut self, speed_action: SpeedAction) {
        match self {
            Self::Unpaused(speed_variant) => speed_variant.regulate(speed_action),
            Self::Paused => {}
        }
    }

    pub fn toggle_pause(&mut self, speed_when_unpaused: Option<SpeedVariant>) {
        match self {
            Self::Unpaused(_) => *self = Self::Paused,
            Self::Paused => *self = Self::Unpaused(speed_when_unpaused.unwrap_or_default()),
        }
    }
}

impl SpeedVariant {
    pub fn to_duration(&self) -> Duration {
        match self {
            SpeedVariant::Slow => Duration::from_millis(500),
            SpeedVariant::Normal => Duration::from_millis(100),
            SpeedVariant::Fast => Duration::from_millis(50),
        }
    }

    pub fn regulate(&mut self, speed_action: SpeedAction) {
        match speed_action {
            SpeedAction::Increase => match self {
                SpeedVariant::Slow => *self = SpeedVariant::Normal,
                SpeedVariant::Normal => *self = SpeedVariant::Fast,
                _ => {}
            },
            SpeedAction::Decrease => match self {
                SpeedVariant::Normal => *self = SpeedVariant::Slow,
                SpeedVariant::Fast => *self = SpeedVariant::Normal,
                _ => {}
            },
        }
    }
}

impl Default for SpeedVariant {
    fn default() -> Self {
        Self::Normal
    }
}

impl Default for Speed {
    fn default() -> Self {
        Self::Unpaused(SpeedVariant::default())
    }
}

impl std::fmt::Display for Speed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Paused => write!(f, "Paused"),
            Self::Unpaused(speed_variant) => match speed_variant {
                SpeedVariant::Slow => write!(f, "Slow"),
                SpeedVariant::Normal => write!(f, "Normal"),
                SpeedVariant::Fast => write!(f, "Fast"),
            },
        }
    }
}

struct GameWidget {
    game: Game,
    speed_when_unpaused: SpeedVariant,
    speed: Speed,
}

impl GameWidget {
    pub fn new(game: Game, speed: Speed) -> Self {
        let speed_when_unpaused = match speed {
            Speed::Paused => SpeedVariant::Normal,
            Speed::Unpaused(speed_variant) => speed_variant,
        };

        Self {
            game,
            speed,
            speed_when_unpaused,
        }
    }
    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn tick(&mut self) {
        self.game.tick();
    }

    pub fn from_file(path: String, speed: Speed) -> Self {
        let game = match Game::from_file(path.clone()) {
            Ok(game) => game,
            Err(error) => {
                let error_message = match error {
                    CellBoardCreationError::FileError => {
                        format!("error opening file '{}', does it exist?", path)
                    }
                    CellBoardCreationError::FormatError(format_error) => match format_error {
                        FormatErrorVariant::EmptyRow => "empty rows are not allowed".to_owned(),
                        FormatErrorVariant::RowLengthMismatch { row_index } => {
                            format!(
                                "row at index {} does not match previous row lengths",
                                row_index
                            )
                        }
                        FormatErrorVariant::UnrecognizedCharacter(c) => {
                            format!("unrecognized character encountered: {}", c)
                        }
                    },
                };
                panic!("{}", error_message);
            }
        };

        Self::new(game, speed)
    }

    pub fn regulate_speed(&mut self, speed_action: SpeedAction) {
        self.speed.regulate(speed_action);
    }

    pub fn toggle_pause(&mut self) {
        if let Speed::Unpaused(speed_variant) = self.speed {
            self.speed_when_unpaused = speed_variant
        }
        self.speed.toggle_pause(Some(self.speed_when_unpaused));
    }

    pub fn speed(&self) -> Speed {
        self.speed
    }
}

impl Widget for &GameWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let board_width = (self.game.width() * 2 + 2) as u16; // 2 chars per cell + 2 for borders
        let board_height = (self.game.height() + 2) as u16; // 1 row per cell + 2 for borders

        let game_area = Rect {
            x: area.x + (area.width.saturating_sub(board_width)) / 2,
            y: area.y + (area.height.saturating_sub(board_height)) / 2,
            width: board_width.min(area.width),
            height: board_height.min(area.height),
        };

        Block::bordered()
            .title(format!(
                "Generation: {} | Speed: {}",
                self.game.generation(),
                self.speed
            ))
            .render(game_area, buf);

        let inner = game_area.inner(Margin::new(1, 1));

        for (Slot(y, x), cell) in self.game.slots_and_cells() {
            let screen_x = inner.x + (x as u16) * 2; // 2 chars wide per cell
            let screen_y = inner.y + y as u16;

            let (symbol, style) = match cell {
                Cell::Alive => ("██", Style::default().fg(Color::White)),
                Cell::Dead => ("  ", Style::default().fg(Color::Black)),
            };

            buf.set_string(screen_x, screen_y, symbol, style);
        }

        let legend_y = game_area.y + game_area.height + 1;
        if legend_y < area.height {
            let legend_text =
                "q: Quit  |  ↑/→: Speed Up  |  ↓/←: Slow Down  |  Space: Pause/Unpause";
            let legend_area = Rect {
                x: area.x + (area.width.saturating_sub(legend_text.len() as u16)) / 2,
                y: legend_y,
                width: legend_text.len() as u16,
                height: 1,
            };

            Paragraph::new(legend_text)
                .style(Style::default().fg(Color::DarkGray))
                .render(legend_area, buf);
        }
    }
}

enum SpeedAction {
    Increase,
    Decrease,
}

enum UserAction {
    Quit,
    TogglePause,
    RegulateSpeed(SpeedAction),
}
enum UpdateEvent {
    Tick,
    Input(UserAction),
}

fn handle_tick(mut speed: Speed, tx: mpsc::Sender<UpdateEvent>, control_rx: mpsc::Receiver<Speed>) {
    loop {
        while let Speed::Unpaused(speed_variant) = speed {
            match control_rx.recv_timeout(speed_variant.to_duration()) {
                Ok(new_speed) => {
                    speed = new_speed;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    tx.send(UpdateEvent::Tick)
                        .expect("mpsc channel to work correctly");
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        while let Speed::Paused = speed {
            speed = control_rx.recv().expect("mpsc channel to work");
        }
    }
}

fn handle_user_input(tx: mpsc::Sender<UpdateEvent>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => {
                            tx.send(UpdateEvent::Input(UserAction::Quit))
                        }
                        KeyCode::Left | KeyCode::Down => tx.send(UpdateEvent::Input(
                            UserAction::RegulateSpeed(SpeedAction::Decrease),
                        )),
                        KeyCode::Right | KeyCode::Up => tx.send(UpdateEvent::Input(
                            UserAction::RegulateSpeed(SpeedAction::Increase),
                        )),
                        KeyCode::Char(' ') => tx.send(UpdateEvent::Input(UserAction::TogglePause)),
                        _ => Ok(()),
                    }
                    .expect("mpsc channel to work correctly")
                }
            }
            _ => {}
        }
    }
}

fn main() -> io::Result<()> {
    let command = CliCommand::parse();

    let CliCommand {
        from_file,
        height,
        width,
        alive_probability,
        speed: speed_variant,
    } = command;

    let speed = Speed::Unpaused(speed_variant);

    let mut game_widget = if let Some(file_path) = from_file {
        GameWidget::from_file(file_path, speed)
    } else {
        GameWidget::new(
            Game::new(height as usize, width as usize).randomize(alive_probability),
            speed,
        )
    };

    let mut terminal = ratatui::init();

    let (update_tx, update_rx) = mpsc::channel::<UpdateEvent>();
    let update_tx_to_tick = update_tx.clone();
    let update_tx_to_user_input = update_tx.clone();

    let (speed_tx, speed_rx) = mpsc::channel::<Speed>();

    thread::spawn(move || {
        handle_tick(speed, update_tx_to_tick, speed_rx);
    });

    thread::spawn(move || {
        handle_user_input(update_tx_to_user_input);
    });

    loop {
        terminal.draw(|frame| game_widget.draw(frame))?;
        match update_rx.recv().unwrap() {
            UpdateEvent::Tick => game_widget.tick(),
            UpdateEvent::Input(user_action) => match user_action {
                UserAction::Quit => break,
                UserAction::RegulateSpeed(speed_action) => {
                    game_widget.regulate_speed(speed_action);
                    speed_tx
                        .send(game_widget.speed())
                        .expect("mpsc channel to work correctly");
                }
                UserAction::TogglePause => {
                    game_widget.toggle_pause();
                    speed_tx
                        .send(game_widget.speed())
                        .expect("mpsc channel to work correctly");
                }
            },
        }
    }
    ratatui::restore();
    Ok(())
}

mod commands {
    use clap::Parser;

    use crate::SpeedVariant;

    #[derive(Parser)]
    pub struct CliCommand {
        // initializes the life board from a .life file with the specified path
        #[arg(short, long)]
        pub from_file: Option<String>,

        // height of the life board
        #[arg(long, default_value_t = 20)]
        pub height: u8,

        // Width of the life board
        #[arg(long, default_value_t = 20)]
        pub width: u8,

        // Probability that a cell will be initialized as alive
        #[arg(long, default_value_t = 0.2)]
        pub alive_probability: f64,

        // Simulation speed
        #[arg(value_enum, long, default_value_t = SpeedVariant::Normal)]
        pub speed: SpeedVariant,
    }
}
