use std::{iter, vec::Vec};

use bf::interpreter::Tape;
use itertools::Itertools;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

use crate::widgets::CellWidget;

pub struct ChunkedTapeWidget(Vec<TapeChunkWidget>);

impl ChunkedTapeWidget {
    pub fn new(tape: &Tape, width: i32, ascii: bool) -> Self {
        let cells = tape.cells();

        // Each cell is 4 wide + the extra vertical separator at the end
        let chunk_size = ((width - 1) / 4) as usize;
        let end_tape = cells.len() - 1;

        let chunks = cells
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
                            is_highlighted: tape_i == tape.cursor(),
                            ascii,
                        }
                    },
                )
            })
            .map(|chunk| TapeChunkWidget::from(chunk.into_iter()))
            .collect();

        Self(chunks)
    }
}

impl ChunkedTapeWidget {
    delegate::delegate! {
        to self.0 {
            pub fn len(&self) -> usize;
        }
    }
}

impl Widget for ChunkedTapeWidget {
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

pub struct TapeChunkWidget(Vec<CellWidget>);

impl<I: Iterator<Item = CellWidget>> From<I> for TapeChunkWidget {
    fn from(value: I) -> Self {
        Self(value.collect())
    }
}

impl TapeChunkWidget {
    pub fn new(tape: &Tape, offset: usize, size: usize, ascii: bool) -> Self {
        let cells = tape.cells();
        let end_tape = cells.len() - 1;
        let end_chunk = (offset + size - 1).min(end_tape);
        let chunk = cells.iter().enumerate().skip(offset).take(size).map(
            |(i, cell)| CellWidget {
                value: cell.value(),
                left_cap: i == 0,
                right_border_cap: if i == end_chunk {
                    Some(i == end_tape)
                } else {
                    None
                },
                is_highlighted: i == tape.cursor(),
                ascii,
            },
        );
        Self(chunk.collect())
    }
}

impl Widget for TapeChunkWidget {
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
