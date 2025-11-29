use std::{thread, time::Duration};

use clap::Parser;

use crate::commands::CliCommand;
use game_of_life::game::Game;

fn main() {
    let command = CliCommand::parse();

    let CliCommand {
        from_file,
        height,
        width,
    } = command;

    let mut game: Game = if let Some(file_path) = from_file {
        todo!()
    } else {
        Game::new(height as usize, width as usize).randomize(0.2)
    };

    loop {
        println!("{}", game.to_string());
        game.tick();
        thread::sleep(Duration::from_millis(100));
    }
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
    }
}
