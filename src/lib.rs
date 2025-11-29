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
    use std::io::{BufRead, BufReader};

    pub struct CellBoard {
        height: usize,
        width: usize,
        cells: Vec<Vec<Cell>>,
    }

    impl CellBoard {
        pub fn new(height: usize, width: usize) -> Self {
            Self {
                height,
                width,
                cells: vec![vec![Cell::Dead; width]; height],
            }
        }

        pub fn from_file(path: String) -> Result<Self, CellBoardCreationError> {
            let file = std::fs::File::open(path)?;
            let reader = BufReader::new(file);
            let mut row_length: Option<usize> = None;
            let mut row_vec = Vec::new();
            for (i, line_res) in reader.lines().enumerate() {
                let mut col_vec = Vec::new();
                let line = line_res?;
                let line_length = line.len();
                match row_length {
                    None => row_length = Some(line_length),
                    Some(length) => {
                        if line_length != length {
                            return Err(CellBoardCreationError::FormatError(
                                FormatErrorVariant::RowLengthMismatch { row_index: i },
                            ));
                        } else if length == 0 {
                            return Err(CellBoardCreationError::FormatError(
                                FormatErrorVariant::EmptyRow,
                            ));
                        }
                    }
                }
                for c in line.chars() {
                    let cell = match c {
                        'X' => Cell::Alive,
                        'O' => Cell::Dead,
                        _ => {
                            return Err(CellBoardCreationError::FormatError(
                                FormatErrorVariant::UnrecognizedCharacter(c),
                            ));
                        }
                    };
                    col_vec.push(cell);
                }
                row_vec.push(col_vec);
            }
            Ok(Self {
                height: row_vec.len(),
                width: row_length.unwrap_or(0),
                cells: row_vec,
            })
        }

        pub fn set_slot(&mut self, slot: Slot, cell: Cell) {
            let Slot(row, col) = slot;
            self.cells[row][col] = cell;
        }

        pub fn get_slot(&self, slot: Slot) -> Cell {
            let Slot(row, col) = slot;
            self.cells[row][col]
        }

        pub fn apply_to_slot(&mut self, slot: Slot, action: Action) {
            let Slot(row, col) = slot;
            self.cells[row][col].apply(action);
        }

        pub fn height(&self) -> usize {
            self.height
        }

        pub fn width(&self) -> usize {
            self.width
        }
    }

    impl ToString for CellBoard {
        fn to_string(&self) -> String {
            let mut string_representation = String::new();
            for row in &self.cells {
                for cell in row {
                    string_representation += match cell {
                        Cell::Dead => " ",
                        Cell::Alive => "X",
                    }
                }
                string_representation += "\n";
            }
            string_representation
        }
    }
    pub struct Game {
        generation: usize,
        cell_board: CellBoard,
    }

    pub enum FormatErrorVariant {
        RowLengthMismatch { row_index: usize },
        UnrecognizedCharacter(char),
        EmptyRow,
    }

    pub enum CellBoardCreationError {
        FileError,
        FormatError(FormatErrorVariant),
    }

    impl From<std::io::Error> for CellBoardCreationError {
        fn from(_: std::io::Error) -> Self {
            Self::FileError
        }
    }

    impl Game {
        pub fn new(height: usize, width: usize) -> Self {
            Self {
                generation: 0,
                cell_board: CellBoard::new(height, width),
            }
        }
        pub fn from_file(path: String) -> Result<Self, CellBoardCreationError> {
            let cell_board = CellBoard::from_file(path)?;
            Ok(Self {
                generation: 0,
                cell_board,
            })
        }
        pub fn randomize(mut self, alive_probability: f64) -> Self {
            for row in 0..self.cell_board.height() {
                for col in 0..self.cell_board.width() {
                    self.cell_board.set_slot(
                        Slot(row, col),
                        if rand::random_bool(alive_probability) {
                            Cell::Alive
                        } else {
                            Cell::Dead
                        },
                    )
                }
            }
            self
        }
        pub fn tick(&mut self) {
            let mut actions_to_apply = HashMap::new();
            for row in 0..self.cell_board.height() {
                for col in 0..self.cell_board.width() {
                    let slot = Slot(row, col);
                    let action = self.get_action(slot);
                    actions_to_apply.insert(slot, action);
                }
            }
            for (slot, action) in actions_to_apply {
                self.cell_board.apply_to_slot(slot, action);
            }
            self.generation += 1;
        }

        fn get_action(&self, slot: Slot) -> Action {
            let Slot(row, col) = slot;
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
                let (new_y, new_x) = (
                    add_mod_n(row, dy, self.cell_board.height()),
                    add_mod_n(col, dx, self.cell_board.width()),
                );
                if let Cell::Alive = self.cell_board.get_slot(Slot(new_y, new_x)) {
                    live_neighbors += 1;
                }
            }
            match self.cell_board.get_slot(slot) {
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
            self.cell_board.apply_to_slot(slot, action);
        }
        pub fn generation(&self) -> usize {
            self.generation
        }
        pub fn slots_and_cells(&self) -> impl Iterator<Item = (Slot, Cell)> {
            (0..self.cell_board.height()).flat_map(move |y| {
                (0..self.cell_board.width()).map(move |x| {
                    let slot = Slot(y, x);
                    (slot, self.cell_board.get_slot(slot))
                })
            })
        }
        pub fn height(&self) -> usize {
            self.cell_board.height()
        }
        pub fn width(&self) -> usize {
            self.cell_board.width()
        }
    }

    const DEFAULT_GAME_SIZE: usize = 8;

    impl Default for Game {
        fn default() -> Self {
            Self::new(DEFAULT_GAME_SIZE, DEFAULT_GAME_SIZE)
        }
    }

    impl ToString for Game {
        fn to_string(&self) -> String {
            let mut string_representation = String::new();
            string_representation += &self.cell_board.to_string();
            string_representation += "\n";
            string_representation += &format!("Generation: {}", self.generation);
            string_representation
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
