// use std::io::{self, Write};

mod cell;
use cell::Cell;

#[derive(Debug)]
pub struct Tape {
    cells: Vec<Cell>,
    cursor: usize,
    lines_printed: u32,
}

impl Tape {
    pub fn new() -> Self {
        Self {
            cells: vec![Cell::new(); 1],
            cursor: 0,
            lines_printed: 0,
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

    pub fn print(&mut self, ascii_only: bool) {
        let print_top_bot = |(left, sep, right, spacer)| {
            print!("{0}{1}{1}{1}", left, spacer);
            self.cells
                .iter()
                .skip(1)
                .for_each(|_| print!("{0}{1}{1}{1}", sep, spacer));
            println!("{}", right);
        };

        let (top_chars, vert_sep, bot_chars, cursor) = if ascii_only {
            let t_b_chars = ('+', '+', '+', '-');
            (t_b_chars, '|', t_b_chars, '^')
        } else {
            (('┌', '┬', '┐', '─'), '│', ('└', '┴', '┘', '─'), '↑')
        };

        // for each line that was printed:
        //   move up 1 line, move to col 0, clear to EOL
        (0..self.lines_printed).for_each(|_| {
            print!("\x1b[1A\r\x1b[K");
        });

        // Top of tape box
        print_top_bot(top_chars);

        // Tape contents and separators
        self.cells
            .iter()
            .for_each(|c| print!("{}{}", vert_sep, c.display()));
        println!("{}", vert_sep);

        // Bottom of tape box
        print_top_bot(bot_chars);

        // Cursor
        println!("{:>1$}", cursor, 3 + self.cursor * 4);

        self.lines_printed = 4;
    }
}
