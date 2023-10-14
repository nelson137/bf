use std::{iter, vec::Vec};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

use crate::widgets::CellWidget;

pub struct ChunkedTapeWidget(Vec<TapeChunkWidget>);

impl<I: Iterator<Item = TapeChunkWidget>> From<I> for ChunkedTapeWidget {
    fn from(value: I) -> Self {
        Self(value.collect())
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
