use std::thread;
use std::time::Duration;

use game_of_life::game::Game;

const SLEEP_TIME: u64 = 100;

fn main() {
    let mut game = Game::new(20, 20);
    // cool recurring pattern with a period of 15
    // game.apply_action(Slot(1, 4), Action::Live);
    // game.apply_action(Slot(1, 5), Action::Live);
    // game.apply_action(Slot(1, 6), Action::Live);

    // game.apply_action(Slot(4, 4), Action::Live);
    // game.apply_action(Slot(4, 5), Action::Live);
    // game.apply_action(Slot(4, 6), Action::Live);

    // game.apply_action(Slot(9, 4), Action::Live);
    // game.apply_action(Slot(9, 5), Action::Live);
    // game.apply_action(Slot(9, 6), Action::Live);

    // game.apply_action(Slot(12, 4), Action::Live);
    // game.apply_action(Slot(12, 5), Action::Live);
    // game.apply_action(Slot(12, 6), Action::Live);

    // game.apply_action(Slot(2, 3), Action::Live);
    // game.apply_action(Slot(3, 3), Action::Live);
    // game.apply_action(Slot(2, 7), Action::Live);
    // game.apply_action(Slot(3, 7), Action::Live);

    // game.apply_action(Slot(10, 3), Action::Live);
    // game.apply_action(Slot(11, 3), Action::Live);
    // game.apply_action(Slot(10, 7), Action::Live);
    // game.apply_action(Slot(11, 7), Action::Live);

    game = game.randomize(0.2);

    loop {
        println!("{}", game.to_string());
        game.tick();
        thread::sleep(Duration::from_millis(SLEEP_TIME));
    }
}
