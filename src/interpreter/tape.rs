use std::{iter, vec::Vec};

use itertools::Itertools;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

use crate::widgets::CellWidget;

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
    ) -> WindowDisplay {
        let end_tape = self.cells.len() - 1;
        let end_chunk = (offset + size - 1).min(end_tape);
        WindowDisplay(
            self.cells
                .iter()
                .map(Cell::value)
                .enumerate()
                .skip(offset)
                .take(size)
                .map(|(i, value)| CellWidget {
                    value,
                    left_cap: i == 0,
                    right_border_cap: if i == end_chunk {
                        Some(i == end_tape)
                    } else {
                        None
                    },
                    is_highlighted: i == self.cursor,
                    ascii,
                })
                .collect::<Vec<_>>(),
        )
    }

    pub fn chunks(&self, width: i32, ascii: bool) -> ChunkedTapeDisplay {
        // Each cell is 4 wide + the extra vertical separator at the end
        let chunk_size = ((width - 1) / 4) as usize;
        let end_tape = self.cells.len() - 1;

        ChunkedTapeDisplay(
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
                .map(|chunk| WindowDisplay(chunk.into_iter().collect()))
                .collect::<Vec<_>>(),
        )
    }
}

pub struct ChunkedTapeDisplay(Vec<WindowDisplay>);

impl ChunkedTapeDisplay {
    delegate::delegate! {
        to self.0 {
            pub fn len(&self) -> usize;
        }
    }
}

impl Widget for ChunkedTapeDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                iter::repeat(Constraint::Length(3))
                    .take(self.0.len())
                    .collect::<Vec<_>>(),
            )
            .split(area);

        for (chunk, &chunk_area) in self.0.into_iter().zip(layout.iter()) {
            chunk.render(chunk_area, buf);
        }
    }
}

pub struct WindowDisplay(Vec<CellWidget>);

impl Widget for WindowDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let len = self.0.len();
        if len == 0 {
            return;
        }

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                iter::repeat(Constraint::Length(4))
                    .take(len - 1)
                    .chain(iter::once(Constraint::Min(0)))
                    .collect::<Vec<Constraint>>(),
            )
            .split(area);

        for (cell, cell_area) in self.0.into_iter().zip(layout.iter()) {
            cell.render(*cell_area, buf);
        }
    }
}
