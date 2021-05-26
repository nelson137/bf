use std::{iter, vec::Vec};

use itertools::Itertools;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

use crate::{tui_util::TAPE_BORDER_SET, util::EOL};

use super::cell::{Cell, CellDisplay};

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

    pub fn window(
        &self,
        offset: usize,
        size: usize,
        ascii: bool
    ) -> WindowDisplay {
        let end_tape = self.cells.len() - 1;
        let end_chunk = (offset + size - 1).min(end_tape);
        WindowDisplay(
            self.cells.iter()
                .enumerate()
                .skip(offset)
                .take(size)
                .map(|(i, c)| CellDisplay::new(
                    c,
                    i == 0,
                    if i == end_chunk { Some(i == end_tape) } else { None },
                    i == self.cursor,
                    ascii,
                ))
                .collect::<Vec<_>>()
        )
    }

    pub fn chunks(&self, width: i32, ascii: bool) -> ChunkedTapeDisplay {
        // Each cell is 4 wide + the extra vertical separator at the end
        let chunk_size = ((width - 1) / 4) as usize;
        let end_tape = self.cells.len() - 1;

        ChunkedTapeDisplay(
            self.cells.iter()
                .enumerate()
                .chunks(chunk_size).into_iter()
                .map(|chunk| {
                    let chunk = chunk.collect::<Vec<_>>();
                    let end_chunk = chunk.len() - 1;
                    chunk.into_iter()
                        .enumerate()
                        .map(move |(chunk_i, (tape_i, cell))| {
                            let right = if chunk_i == end_chunk {
                                Some(tape_i == end_tape)
                            } else {
                                None
                            };
                            CellDisplay::new(
                                cell,
                                tape_i == 0,
                                right,
                                tape_i == self.cursor,
                                ascii,
                            )
                        })
                })
                .map(|chunk| WindowDisplay(chunk.into_iter().collect()))
                .collect::<Vec<_>>(),
        )
    }

}

pub struct ChunkedTapeDisplay<'a>(Vec<WindowDisplay<'a>>);

impl<'a> ChunkedTapeDisplay<'a> {

    pub fn display(&mut self, prefix: &str) -> String {
        self.0.iter()
            .map(|chunk| chunk.display(prefix))
            .collect()
    }

}

pub struct WindowDisplay<'a>(Vec<CellDisplay<'a>>);

impl<'a> WindowDisplay<'a> {

    fn display_top(&self) -> String {
        self.0.iter().map(|cell| cell.display_top()).collect::<String>()
    }

    fn display_bottom(&self) -> String {
        self.0.iter().map(|cell| cell.display_bottom()).collect::<String>()
    }

    fn display(&self, prefix: &str) -> String {
        let mut buf = String::new();

        // Top lid
        buf.push_str(prefix);
        buf.push_str(&self.display_top());
        buf.push_str(EOL);

        // Values and separators
        buf.push_str(prefix);
        for cell in self.0.iter() {
            buf.push_str(TAPE_BORDER_SET.vertical);
            if cell.is_highlighted() {
                buf.push_str("\x1b[30m\x1b[46m");
            }
            buf.push_str(&cell.display_value());
            if cell.is_highlighted() {
                buf.push_str("\x1b[0m");
            }
        }
        buf.push_str(TAPE_BORDER_SET.vertical);
        buf.push_str(EOL);

        // Bottom lid
        buf.push_str(prefix);
        buf.push_str(&self.display_bottom());
        buf.push_str(EOL);

        buf
    }

}

impl<'a> Widget for WindowDisplay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let len = self.0.len();
        if len == 0 {
            return;
        }

        let cell_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                iter::repeat(Constraint::Length(4))
                    .take(len - 1)
                    .chain(iter::once(Constraint::Min(0)))
                    .collect::<Vec<Constraint>>()
            )
            .split(area);

        for (cell, cell_area) in self.0.into_iter().zip(cell_areas) {
            cell.render(cell_area, buf);
        }
    }
}
