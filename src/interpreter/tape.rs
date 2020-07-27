use std::vec::Vec;

use itertools::Itertools;
use pancurses::{has_colors, Window, A_UNDERLINE};

use crate::util::{BoxLid, Style, EOL, TAPE_UNICODE};

use super::cell::{Cell, CellDisplay};

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

    pub fn chunks(&self, width: i32) -> ChunkedTape {
        // Each cell is 4 wide + the extra vertical separator
        let cells_per_chunk = ((width - 1) / 4) as usize;
        ChunkedTape::new(&self.cells, cells_per_chunk, self.cursor)
    }
}

pub struct ChunkedTape<'a> {
    chunks: Vec<TapeChunkDisplay<'a>>,
    cursor: (usize, usize),
}

impl<'a> ChunkedTape<'a> {
    pub fn new(cells: &'a [Cell], size: usize, cursor: usize) -> Self {
        let n_chunks = (cells.len() as f64 / size as f64).ceil() as usize;
        let mut chunks = Vec::with_capacity(n_chunks);
        for (i, c) in cells.iter().chunks(size).into_iter().enumerate() {
            chunks.push(TapeChunkDisplay::new(c, i == 0, i == n_chunks - 1));
        }
        Self {
            chunks,
            cursor: (cursor / size, cursor % size),
        }
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn display(&mut self, prefix: &str, ascii_values: bool) -> String {
        let (cursor_y, cursor_x) = self.cursor;
        self.chunks
            .iter_mut()
            .enumerate()
            .map(|(chunk_i, chunk)| {
                for (i, cell_disp) in chunk.iter_mut().enumerate() {
                    if chunk_i == cursor_y && i == cursor_x {
                        cell_disp.highlight();
                    }
                }
                chunk.display(prefix, ascii_values)
            })
            .collect::<String>()
    }

    pub fn nc_display(
        &mut self,
        window: &Window,
        prefix: &str,
        ascii_values: bool,
    ) {
        let (cursor_y, cursor_x) = self.cursor;
        for (chunk_i, chunk) in self.chunks.iter_mut().enumerate() {
            for (i, cell_disp) in chunk.iter_mut().enumerate() {
                if chunk_i == cursor_y && i == cursor_x {
                    cell_disp.highlight();
                }
            }
            chunk.nc_display(window, prefix, ascii_values);
        }
    }
}

pub struct TapeChunkDisplay<'a> {
    pub chunk: Vec<CellDisplay<'a>>,
    left_cap: bool,
    right_cap: bool,
}

impl<'a> TapeChunkDisplay<'a> {
    pub fn new<II: IntoIterator<Item = &'a Cell>>(
        iter: II,
        left_cap: bool,
        right_cap: bool,
    ) -> Self {
        let mut chunk = Vec::new();
        for c in iter {
            chunk.push(CellDisplay::new(&c));
        }
        Self {
            chunk,
            left_cap,
            right_cap,
        }
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut CellDisplay<'a>> {
        self.chunk.iter_mut()
    }

    pub fn display_lid(&self, lid: &BoxLid) -> String {
        self.chunk
            .iter()
            .enumerate()
            .map(|(i, cell)| {
                let right =
                    Some(self.right_cap).filter(|_| i == self.chunk.len() - 1);
                cell.display_lid(lid, i == 0 && self.left_cap, right)
            })
            .collect::<String>()
    }

    pub fn display(&self, prefix: &str, ascii_values: bool) -> String {
        let mut buf = String::new();

        // Top lid
        buf.push_str(prefix);
        buf.push_str(&self.display_lid(&TAPE_UNICODE.top));
        buf.push_str(EOL);

        // Values and separators
        buf.push_str(prefix);
        for cell in self.chunk.iter() {
            buf.push(TAPE_UNICODE.vert_sep);
            if cell.is_highlighted() {
                buf.push_str("\x1b[30m\x1b[46m");
            }
            buf.push_str(&cell.display(ascii_values));
            if cell.is_highlighted() {
                buf.push_str("\x1b[0m");
            }
        }
        buf.push(TAPE_UNICODE.vert_sep);
        buf.push_str(EOL);

        // Bottom lid
        buf.push_str(prefix);
        buf.push_str(&self.display_lid(&TAPE_UNICODE.bot));
        buf.push_str(EOL);

        buf
    }

    pub fn nc_display(
        &self,
        window: &Window,
        prefix: &str,
        ascii_values: bool,
    ) {
        let cursor_style = || {
            if has_colors() {
                Style::Cursor.get()
            } else {
                A_UNDERLINE
            }
        };

        // Top lid
        window.printw(prefix);
        window.printw(self.display_lid(&TAPE_UNICODE.top));
        window.printw(EOL.to_string());

        // Values and separators
        window.printw(prefix);
        for cell in self.chunk.iter() {
            window.printw(TAPE_UNICODE.vert_sep.to_string());
            if cell.is_highlighted() {
                window.attron(cursor_style());
            }
            window.printw(cell.display(ascii_values));
            if cell.is_highlighted() {
                window.attroff(cursor_style());
            }
        }
        window.printw(TAPE_UNICODE.vert_sep.to_string());
        window.printw(EOL.to_string());

        // Bottom lid
        window.printw(prefix);
        window.printw(self.display_lid(&TAPE_UNICODE.bot));
        window.printw(EOL.to_string());
    }
}
