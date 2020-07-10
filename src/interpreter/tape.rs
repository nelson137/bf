use std::iter::FromIterator;

use itertools::Itertools;
use num_integer::div_rem;

use crate::util::{BoxLid, BOX_UNICODE};

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

    pub fn display(&mut self, width: u32, ascii_values: bool) -> String {
        // Each cell is 4 wide + the extra vertical separator
        let cells_per_chunk = ((width - 1) / 4) as usize;
        let n_chunks =
            (self.cells.len() as f64 / cells_per_chunk as f64).ceil() as usize;
        let (cursor_chunk, cursor_chunk_i) =
            div_rem(self.cursor, cells_per_chunk);
        let cursor_i = Some(cursor_chunk_i);

        self.cells
            .iter()
            .chunks(cells_per_chunk)
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| {
                chunk.collect::<TapeChunk>().display(
                    cursor_i.filter(|_| i == cursor_chunk),
                    i == 0,
                    i == n_chunks - 1,
                    ascii_values,
                )
            })
            .collect::<String>()
    }
}

struct TapeChunk<'a> {
    chunk: Vec<&'a Cell>,
}

impl TapeChunk<'_> {
    pub fn display(
        &self,
        cursor: Option<usize>,
        left_cap: bool,
        right_cap: bool,
        ascii_values: bool,
    ) -> String {
        let mut buf = String::new();

        let display_lid = |buf: &mut String, lid: &BoxLid| {
            let spacer = lid.spacer.to_string().repeat(3);

            // First cell lid
            buf.push(if left_cap { lid.left } else { lid.sep });
            buf.push_str(&spacer);

            // Remaining cell lids
            for _ in 1..self.chunk.len() {
                buf.push(lid.sep);
                buf.push_str(&spacer);
            }

            // Final separator and newline
            buf.push(if right_cap { lid.right } else { lid.sep });
            buf.push('\n');
        };

        // Top lid
        display_lid(&mut buf, &BOX_UNICODE.top);

        // Cell values and separators
        for (i, cell) in self.chunk.iter().enumerate() {
            buf.push(BOX_UNICODE.vert_sep);
            buf.push_str(
                &cell.display(
                    cursor.filter(|c| i == *c).is_some(),
                    ascii_values,
                ),
            );
        }
        buf.push(BOX_UNICODE.vert_sep);
        buf.push('\n');

        // Bottom lid
        display_lid(&mut buf, &BOX_UNICODE.bot);

        buf
    }
}

impl<'a> FromIterator<&'a Cell> for TapeChunk<'a> {
    fn from_iter<T: IntoIterator<Item = &'a Cell>>(iter: T) -> Self {
        Self {
            chunk: iter.into_iter().collect::<Vec<&'a Cell>>(),
        }
    }
}
