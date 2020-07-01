use itertools::Itertools;
use num_integer::div_rem;

use crate::util::{BOX_CHARS_ASCII, BOX_CHARS_UNICODE};

mod cell;
use cell::Cell;

#[derive(Debug)]
pub struct Tape {
    cells: Vec<Cell>,
    cursor: usize,
    lines_printed: usize,
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

    pub fn draw(&mut self, width: u32, ascii_only: bool) {
        let (box_chars, cursor) = if ascii_only {
            (BOX_CHARS_ASCII, '^')
        } else {
            (BOX_CHARS_UNICODE, '↑')
        };

        // for each line that was printed:
        //   move up 1 line, move to col 0, clear to EOL
        for _ in 0..self.lines_printed {
            print!("\x1b[1A\r\x1b[K");
        }

        let chunk_size = ((width - 1) / 4) as usize;
        let mut chunk_i = 0;
        let (cursor_chunk, cursor_chunk_i) = div_rem(self.cursor, chunk_size);

        for chunk in &self.cells.iter().chunks(chunk_size) {
            // Print tape
            let chunk: Vec<_> = chunk.map(|c| c.display()).collect();
            box_chars.draw(&chunk);

            // Print cursor
            if chunk_i == cursor_chunk {
                print!("{:>1$}", cursor, 3 + cursor_chunk_i * 4);
            }
            println!();

            chunk_i += 1;
        }

        self.lines_printed = chunk_i * 4;
    }
}
