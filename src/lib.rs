pub mod utils {
    pub fn add_mod_n(lhs: usize, rhs: i32, modulo: usize) -> usize {
        ((lhs as i32) + rhs)
            .rem_euclid(modulo as i32)
            .try_into()
            .expect("all inputs are usize")
    }
}

pub mod game {
    use self::cell::{Action, Cell, Slot};
    use crate::utils::add_mod_n;
    use std::collections::HashMap;
    pub struct Game {
        cells: Vec<Vec<Cell>>,
    }

    impl Game {
        pub fn print(&self) {
            for row in &self.cells {
                for cell in row {
                    match cell {
                        Cell::Dead => print!(" "),
                        Cell::Alive => print!("X"),
                    }
                }
                print!("\n");
            }
        }
        pub fn new(height: usize, width: usize) -> Self {
            Self {
                cells: vec![vec![Cell::Dead; width]; height],
            }
        }
        pub fn randomize(&mut self, alive_probability: f64) {
            for row in 0..self.cells.len() {
                for col in 0..self.cells[0].len() {
                    self.cells[row][col] = if rand::random_bool(alive_probability) {
                        Cell::Alive
                    } else {
                        Cell::Dead
                    }
                }
            }
        }
        pub fn tick(&mut self) {
            let mut actions_to_apply = HashMap::new();
            for row in 0..self.cells.len() {
                for col in 0..self.cells[0].len() {
                    let slot = Slot(row, col);
                    let action = self.get_action(slot);
                    actions_to_apply.insert(slot, action);
                }
            }
            for (Slot(row, col), action) in actions_to_apply {
                self.cells[row][col].apply(action);
            }
        }

        fn get_action(&self, slot: Slot) -> Action {
            let Slot(row, col) = slot;
            let modulo = self.cells.len();
            let mut live_neighbors: usize = 0;
            for (dy, dx) in [
                (0, 1),
                (-1_i32, 1),
                (-1, 0),
                (-1, -1_i32),
                (0, -1),
                (1, -1),
                (1, 0),
                (1, 1),
            ] {
                let (new_y, new_x) = (add_mod_n(row, dy, modulo), add_mod_n(col, dx, modulo));
                if let Cell::Alive = &self.cells[new_y as usize][new_x as usize] {
                    live_neighbors += 1;
                }
            }
            match &self.cells[row as usize][col as usize] {
                Cell::Alive => match live_neighbors {
                    2..=3 => Action::Live,
                    0..=1 | 4.. => Action::Die,
                },
                Cell::Dead => match live_neighbors {
                    3 => Action::Live,
                    _ => Action::Die,
                },
            }
        }
        pub fn apply_action(&mut self, slot: Slot, action: Action) {
            let Slot(row, col) = slot;
            self.cells[row as usize][col as usize].apply(action);
        }
    }

    const DEFAULT_GAME_SIZE: usize = 8;

    impl Default for Game {
        fn default() -> Self {
            Self::new(DEFAULT_GAME_SIZE, DEFAULT_GAME_SIZE)
        }
    }

    pub mod cell {
        #[derive(Clone, Copy)]
        pub enum Cell {
            Dead,
            Alive,
        }

        impl Cell {
            pub fn apply(&mut self, action: Action) {
                match action {
                    Action::Die => *self = Cell::Dead,
                    Action::Live => *self = Cell::Alive,
                }
            }
        }

        #[derive(Debug)]
        pub enum Action {
            Die,
            Live,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct Slot(pub usize, pub usize);
    }
}
