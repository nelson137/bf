mod cell;
use cell::Cell;

#[derive(Debug)]
pub struct Tape {
    cells: Vec<Cell>,
    cursor: usize,
}

impl Tape {
    pub fn new() -> Self {
        Self {
            cells: vec![Cell::new(); 1],
            cursor: 0,
        }
    }

    fn get(&mut self, index: usize) -> &mut Cell {
        while index > self.cells.len() - 1 {
            self.cells.push(Cell::new());
        }
        unsafe { self.cells.get_unchecked_mut(index) }
    }

    pub fn current(&mut self) -> &mut Cell {
        self.get(self.cursor)
    }

    pub fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn right(&mut self) {
        self.cursor += 1;
        // Force tape to be extended
        self.current();
    }

    pub fn print(&self) {
        let print_top_bot = || self.cells.iter().for_each(|_| print!("+---"));

        // Top of tape box
        print_top_bot();
        println!("+");

        // Tape contents and separators
        self.cells.iter().for_each(|c| print!("|{}", c.display()));
        println!("|");

        // Bottom of tape box
        print_top_bot();
        println!("+");

        // Cursor
        println!("{:>1$}", "^", 3 + self.cursor * 4);
    }
}
