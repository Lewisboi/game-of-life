use std::io;
use std::sync::mpsc;
use std::{thread, time::Duration};

use clap::Parser;
use game_of_life::game::cell::Cell;

use crate::commands::CliCommand;
use crossterm::event::{KeyCode, KeyEventKind};
use game_of_life::game::{Game, cell::Slot};
use ratatui::widgets::{Block, Widget};
use ratatui::{crossterm, prelude::*};

struct GameWidget {
    pub game: Game,
}

impl GameWidget {
    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn tick(&mut self) {
        self.game.tick();
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
            .title(format!("Generation: {}", self.game.generation()))
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
    }
}

enum UserAction {
    Quit,
}
enum UpdateEvent {
    Tick,
    Input(UserAction),
}

fn handle_tick(duration: Duration, tx: mpsc::Sender<UpdateEvent>) {
    loop {
        thread::sleep(duration);
        tx.send(UpdateEvent::Tick)
            .expect("mpsc channel to work correctly");
    }
}

fn handle_user_input(tx: mpsc::Sender<UpdateEvent>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => {
                if key_event.kind == KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => tx
                            .send(UpdateEvent::Input(UserAction::Quit))
                            .expect("mpsc channel to work correctly"),
                        _ => {}
                    }
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
        sleep_interval,
    } = command;

    let mut game: GameWidget = GameWidget {
        game: if let Some(file_path) = from_file {
            todo!()
        } else {
            Game::new(height as usize, width as usize).randomize(alive_probability)
        },
    };

    let mut terminal = ratatui::init();

    let (tx, rx) = mpsc::channel::<UpdateEvent>();
    let tx_to_tick = tx.clone();
    let tx_to_user_input = tx.clone();

    thread::spawn(move || {
        handle_tick(Duration::from_millis(sleep_interval), tx_to_tick);
    });

    thread::spawn(move || {
        handle_user_input(tx_to_user_input);
    });

    loop {
        terminal.draw(|frame| game.draw(frame))?;
        match rx.recv().unwrap() {
            UpdateEvent::Tick => game.tick(),
            UpdateEvent::Input(user_action) => match user_action {
                UserAction::Quit => break,
            },
        }
    }
    ratatui::restore();
    Ok(())
}

mod commands {
    use clap::Parser;

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

        // Simulation sleep interval
        #[arg(long, default_value_t = 100)]
        pub sleep_interval: u64,
    }
}
