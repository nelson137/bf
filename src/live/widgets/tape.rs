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
        /*
         * tape length = 14
         * viewport width = 12
         * cursor margin = 2
         *
         *     vp_begin = 1                          vp_end = 13
         *     ▽                                               ▽
         *     ╔═══════════════════════════════════════════════╗
         *     ║               cursor = 5, vp_cursor = 4       ║
         *     ║               ▽                               ║
         *     ║       ╔═══════════════════════════════╗       ║
         * ┌───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┬───┐
         * │ 0 │ 1 │ 2 │ 3 │ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │10 │11 │12 │13 │
         * └───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┴───┘
         *     ║       ╚═══════════════════════════════╝       ║
         *     ║       △                               △       ║
         *     ║ cursorbox_begin = 2        cursorbox_end = 10 ║
         *     ╚═══════════════════════════════════════════════╝
         *
         * index:
         * 0   1   2   3   4   5   6   7   8   9   10  11  12  13  14
         */

        const CURSOR_MARGIN: usize = 3;

        // The number of cells that can fit within the viewport.
        let vp_width = ((area.width - 1) as f32 / 4.0).floor() as usize;
        // The cell index of the beginning of the viewport.
        let vp_begin = state.viewport_start;
        // The cell index of the end of the viewport, this be past the end of
        // the tape.
        let vp_end = vp_begin + vp_width;
        // The index of the cursor relative to the beginning of the viewport.
        let vp_cursor = self.tape.cursor().saturating_sub(vp_begin);

        // The index of the beginning of the cursorbox relative to the beginning
        // of the viewport.
        let cursorbox_begin = CURSOR_MARGIN;
        // The index of the end of the cursorbox relative to the beginning of
        // the viewport.
        let cursorbox_end = vp_width
            .saturating_sub(CURSOR_MARGIN)
            .max(cursorbox_begin + 1);

        if vp_begin > 0 && self.tape.cursor() < vp_begin + cursorbox_begin {
            // Shift the viewport left to keep the cursor within the cursorbox.
            state.viewport_start -=
                vp_begin.min(vp_begin + cursorbox_begin - self.tape.cursor());
        } else if vp_begin > 0 && vp_end > self.tape.len() {
            // Shift the viewport left to fill the gap at the end.
            state.viewport_start =
                vp_begin.saturating_sub(vp_end - self.tape.len());
        } else if vp_cursor >= cursorbox_end && vp_end < self.tape.len() {
            // Shift the viewport right to keep the cursor within the cursorbox.
            state.viewport_start +=
                (vp_cursor - cursorbox_end + 1).min(self.tape.len() - vp_end);
        }

        self.tape
            .window(state.viewport_start, vp_width, state.ascii_values)
            .render(area, buf);
    }
}
