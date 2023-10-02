use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::interpreter::Tape;

#[derive(Default)]
pub struct TapeViewportState {
    pub viewport_start: usize,
    pub ascii_values: bool,
}

impl TapeViewportState {
    pub fn new(ascii_values: bool) -> Self {
        Self {
            viewport_start: 0,
            ascii_values,
        }
    }
}

pub struct TapeViewport<'tape> {
    tape: &'tape Tape,
}

impl<'tape> TapeViewport<'tape> {
    pub fn new(tape: &'tape Tape) -> Self {
        Self { tape }
    }
}

impl StatefulWidget for TapeViewport<'_> {
    type State = TapeViewportState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let max_cells = (area.width as f32 / 4.0).ceil() as usize;
        let cursor_min = state.viewport_start + 3;
        let cursor_max = state.viewport_start + max_cells.saturating_sub(3);

        if self.tape.cursor() < cursor_min && state.viewport_start > 0 {
            state.viewport_start = state
                .viewport_start
                .saturating_sub(cursor_min - self.tape.cursor());
        } else if self.tape.cursor() > cursor_max {
            state.viewport_start += self.tape.cursor() - cursor_max;
        }

        self.tape
            .window(state.viewport_start, max_cells, state.ascii_values)
            .render(area, buf);
    }
}
