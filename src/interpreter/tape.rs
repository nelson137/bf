use std::vec::Vec;

use itertools::Itertools;

use crate::widgets::{CellWidget, ChunkedTapeWidget, TapeChunkWidget};

use super::cell::Cell;

#[derive(Clone, Debug)]
pub struct Tape {
    cells: Vec<Cell>,
    cursor: usize,
}

impl Default for Tape {
    fn default() -> Self {
        Self {
            cells: vec![Cell::new(); 1],
            cursor: 0,
        }
    }
}

impl Tape {
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    fn get(&mut self, index: usize) -> &mut Cell {
        while index > self.cells.len() - 1 {
            self.cells.push(Cell::new());
        }
        &mut self.cells[index]
    }

    pub fn current(&mut self) -> &mut Cell {
        self.get(self.cursor)
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor += 1;
        // Force tape to be extended
        self.current();
    }

    pub fn window(
        &self,
        offset: usize,
        size: usize,
        ascii: bool,
    ) -> TapeChunkWidget {
        let end_tape = self.cells.len() - 1;
        let end_chunk = (offset + size - 1).min(end_tape);
        self.cells
            .iter()
            .enumerate()
            .skip(offset)
            .take(size)
            .map(|(i, cell)| CellWidget {
                value: cell.value(),
                left_cap: i == 0,
                right_border_cap: if i == end_chunk {
                    Some(i == end_tape)
                } else {
                    None
                },
                is_highlighted: i == self.cursor,
                ascii,
            })
            .into()
    }

    pub fn chunks(&self, width: i32, ascii: bool) -> ChunkedTapeWidget {
        // Each cell is 4 wide + the extra vertical separator at the end
        let chunk_size = ((width - 1) / 4) as usize;
        let end_tape = self.cells.len() - 1;

        self.cells
            .iter()
            .enumerate()
            .chunks(chunk_size)
            .into_iter()
            .map(|chunk| {
                let chunk = chunk.collect::<Vec<_>>();
                let end_chunk = chunk.len() - 1;
                chunk.into_iter().enumerate().map(
                    move |(chunk_i, (tape_i, cell))| {
                        let right_border_cap = if chunk_i == end_chunk {
                            Some(tape_i == end_tape)
                        } else {
                            None
                        };
                        CellWidget {
                            value: cell.value(),
                            left_cap: tape_i == 0,
                            right_border_cap,
                            is_highlighted: tape_i == self.cursor,
                            ascii,
                        }
                    },
                )
            })
            .map(|chunk| TapeChunkWidget::from(chunk.into_iter()))
            .into()
    }
}
