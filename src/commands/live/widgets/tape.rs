use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{StatefulWidget, Widget},
};

use crate::interpreter::Tape;

#[derive(Clone, Copy, Default)]
pub struct TapeViewportState {
    pub offset: usize,
    pub ascii_values: bool,
}

impl TapeViewportState {
    pub fn new(ascii_values: bool) -> Self {
        Self {
            offset: 0,
            ascii_values,
        }
    }

    #[cfg(test)]
    fn _new(offset: usize) -> Self {
        Self {
            offset,
            ascii_values: false,
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
         * viewport offset = 1
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

        // The number of cells that can fit within the viewport.
        let vp_width = ((area.width - 1) as f32 / 4.0).floor() as usize;
        // The cell index of the beginning of the viewport.
        let vp_begin = state.offset;
        // The cell index of the end of the viewport, this be past the end of
        // the tape.
        let vp_end = vp_begin + vp_width;
        // The index of the cursor relative to the beginning of the viewport.
        let vp_cursor = self.tape.cursor().saturating_sub(vp_begin);

        // The number of cells to keep between the cursor and either edge of the
        // viewport. Only applicable when the cursor is at least that many cells
        // away from either end of the tape.
        let cursor_margin = match vp_width {
            0..=5 => 0,
            6..=10 => 1,
            11..=15 => 2,
            _ => 3,
        };

        // The index of the beginning of the cursorbox relative to the beginning
        // of the viewport.
        let cursorbox_begin = cursor_margin;
        // The index of the end of the cursorbox relative to the beginning of
        // the viewport.
        let cursorbox_end = vp_width
            .saturating_sub(cursor_margin)
            .max(cursorbox_begin + 1);

        if vp_begin > 0 && self.tape.cursor() < vp_begin + cursorbox_begin {
            // Shift the viewport left to keep the cursor within the cursorbox.
            state.offset -=
                vp_begin.min(vp_begin + cursorbox_begin - self.tape.cursor());
        } else if vp_begin > 0 && vp_end > self.tape.len() {
            // Shift the viewport left to fill the gap at the end.
            state.offset = vp_begin.saturating_sub(vp_end - self.tape.len());
        } else if vp_cursor >= cursorbox_end && vp_end < self.tape.len() {
            // Shift the viewport right to keep the cursor within the cursorbox.
            state.offset +=
                (vp_cursor - cursorbox_end + 1).min(self.tape.len() - vp_end);
        }

        self.tape
            .window(state.offset, vp_width, state.ascii_values)
            .render(area, buf);
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use crate::util::test::tape_from_script;

    use super::*;

    mod utils {
        use std::iter;

        use itertools::Itertools;
        use ratatui::{style::Style, Terminal};

        use crate::util::test::{
            terminal, MyTestBackend, CELL_STYLE_CURSOR, CELL_STYLE_NORMAL,
        };

        use super::super::*;

        pub fn terminal_for_tape(
            width: u16,
        ) -> (Terminal<MyTestBackend>, MyTestBackend) {
            terminal(1 + 4 * width, 3)
        }

        pub fn render_tape(
            term: &mut Terminal<MyTestBackend>,
            widget: TapeViewport,
            state: &mut TapeViewportState,
        ) {
            term.draw(|f| f.render_stateful_widget(widget, f.size(), state))
                .unwrap();
        }

        pub trait TapeBufferTestExts {
            fn set_tape_content(
                &mut self,
                cells: impl IntoIterator<Item = (u8, Style)>,
            );
        }

        impl TapeBufferTestExts for Buffer {
            fn set_tape_content(
                &mut self,
                cells: impl IntoIterator<Item = (u8, Style)>,
            ) {
                for (i, (value, style)) in cells.into_iter().enumerate() {
                    let x = 1 + 4 * i as u16;
                    self.set_string(x, 1, format!("{value:^3}"), style);
                }
            }
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum TapeEndcaps {
            None,
            Left,
            Right,
            LeftRight,
        }

        impl TapeEndcaps {
            pub fn left(self) -> bool {
                matches!(self, TapeEndcaps::Left | TapeEndcaps::LeftRight)
            }

            pub fn right(self) -> bool {
                matches!(self, TapeEndcaps::Right | TapeEndcaps::LeftRight)
            }
        }

        #[rustfmt::skip]
        fn tape_horizontal_line(
            endcaps: TapeEndcaps,
            left_cap: &str,
            middle_cap: &str,
            right_cap: &str,
            middle: &str,
            len: usize,
        ) -> String {
            let left = if endcaps.left() { left_cap } else { middle_cap };
            let middle = iter::repeat(middle).take(len).join(middle_cap);
            let right = if endcaps.right() { right_cap } else { middle_cap };
            String::with_capacity(len * 4 + 1) + left + &middle + right
        }

        pub fn buf_for_tape_viewport(
            endcaps: TapeEndcaps,
            vp_cursor: usize,
            cells: &[u8],
        ) -> Buffer {
            let len = cells.len();
            let top = tape_horizontal_line(endcaps, "┌", "┬", "┐", "───", len);
            let middle =
                tape_horizontal_line(endcaps, "│", "│", "│", "   ", len);
            let bottom =
                tape_horizontal_line(endcaps, "└", "┴", "┘", "───", len);

            let mut buffer = Buffer::with_lines(vec![top, middle, bottom]);

            let contents =
                cells.iter().copied().enumerate().map(|(i, value)| {
                    let style = if i == vp_cursor {
                        CELL_STYLE_CURSOR
                    } else {
                        CELL_STYLE_NORMAL
                    };
                    (value, style)
                });
            buffer.set_tape_content(contents);

            buffer
        }
    }
    use utils::*;

    #[test]
    fn state_ctor_sets_offset_to_0() {
        let state = TapeViewportState::new(false);
        assert_eq!(state.offset, 0);
    }

    #[test_case(true  ; "sets ascii to true when given true")]
    #[test_case(false ; "sets ascii to false when given false")]
    fn state_ctor(ascii: bool) {
        let state = TapeViewportState::new(ascii);
        assert_eq!(state.ascii_values, ascii);
    }

    #[test]
    fn renders_a_default_tape() {
        let (mut term, backend) = terminal_for_tape(1);

        let tape = Tape::default();
        let widget = TapeViewport::new(&tape);
        let mut state = TapeViewportState::default();

        let expected_buf =
            buf_for_tape_viewport(TapeEndcaps::LeftRight, 0, &[0]);

        render_tape(&mut term, widget, &mut state);

        backend.get().assert_buffer(&expected_buf);
    }

    #[test]
    fn renders_a_script() {
        let (mut term, backend) = terminal_for_tape(3);

        /*
         * Script:
         * Print a capital 'A' and a newline.
         */
        let script = "++++++++[>+>++++++++<<-]>++>+.<.";
        let tape = tape_from_script(script);
        let widget = TapeViewport::new(&tape);
        let mut state = TapeViewportState::default();

        let expected_buf =
            buf_for_tape_viewport(TapeEndcaps::LeftRight, 1, &[0, 10, 65]);

        render_tape(&mut term, widget, &mut state);

        backend.get().assert_buffer(&expected_buf);
    }

    #[test_case(3  ; "for cursor margin 0")]
    #[test_case(8  ; "for cursor margin 1")]
    #[test_case(12 ; "for cursor margin 2")]
    #[test_case(16 ; "for cursor margin 3")]
    fn shifts_right_when_cursor_is_right_of_the_viewport(width: usize) {
        let (mut term, backend) = terminal_for_tape(width as u16);

        /*
         * Script:
         * Move right the same number of times as the width of the viewport so
         * that the tape one cell longer than the viewport and shifting it right
         * by one. The cursor margin (AKA cursorbox) is ignored because
         * the end of viewport is also the end of the tape.
         */
        let script = ">".repeat(width);
        let tape = tape_from_script(&script);
        let widget = TapeViewport::new(&tape);
        let mut state = TapeViewportState::default();

        let cells = vec![0; width];
        let expected_buf =
            buf_for_tape_viewport(TapeEndcaps::Right, width - 1, &cells);

        render_tape(&mut term, widget, &mut state);

        assert_eq!(state.offset, 1);
        backend.get().assert_buffer(&expected_buf);
    }

    #[test_case(3,  0 ; "for cursor margin 0")]
    #[test_case(8,  1 ; "for cursor margin 1")]
    #[test_case(12, 2 ; "for cursor margin 2")]
    #[test_case(16, 3 ; "for cursor margin 3")]
    fn shifts_right_when_cursor_is_right_of_the_cursorbox(
        width: usize,
        margin: usize,
    ) {
        let (mut term, backend) = terminal_for_tape(width as u16);

        /*
         * Script:
         * - Move right to make the tape longer than the viewport.
         * - Move left to the beginning of the tape.
         * - Move right so that the cursor one past the end of the cursorbox.
         */
        let script = ">".repeat(width * 2)
            + &"<".repeat(width * 2)
            + &">".repeat(width - margin);
        let tape = tape_from_script(&script);
        let widget = TapeViewport::new(&tape);
        let mut state = TapeViewportState::default();

        let expected_buf = buf_for_tape_viewport(
            TapeEndcaps::None,
            width - margin - 1,
            &vec![0; width],
        );

        render_tape(&mut term, widget, &mut state);

        assert_eq!(state.offset, 1);
        backend.get().assert_buffer(&expected_buf);
    }

    #[test]
    fn shifts_left_when_cursor_is_left_of_the_viewport() {
        let width = 3_usize;
        let (mut term, backend) = terminal_for_tape(width as u16);

        /*
         * Script:
         * - Move right the same number of times as the width of the viewport,
         *   making the tape one longer than the viewport and shifting it right
         *   by one.
         * - Move left so that the cursor is at the beginning of the tape and
         *   one before the beginning of the viewport.
         */
        let script = ">".repeat(width) + &"<".repeat(width);
        let tape = tape_from_script(&script);
        let widget = TapeViewport::new(&tape);
        let mut state = TapeViewportState::_new(1);

        let cells = vec![0; width];
        let expected_buf = buf_for_tape_viewport(TapeEndcaps::Left, 0, &cells);

        render_tape(&mut term, widget, &mut state);

        assert_eq!(state.offset, 0);
        backend.get().assert_buffer(&expected_buf);
    }

    #[test_case(3,  0 ; "for cursor margin 0")]
    #[test_case(8,  1 ; "for cursor margin 1")]
    #[test_case(12, 2 ; "for cursor margin 2")]
    #[test_case(16, 3 ; "for cursor margin 3")]
    fn shifts_left_when_cursor_is_left_of_the_cursorbox(
        width: usize,
        margin: usize,
    ) {
        let (mut term, backend) = terminal_for_tape(width as u16);

        /*
         * Script:
         * - Move right so that the tape is longer than the viewport.
         * - Move left so that the cursor is one before the beginning of the
         *   cursorbox.
         */
        let script = ">".repeat(width * 2) + &"<".repeat(width - margin);
        let tape = tape_from_script(&script);
        let widget = TapeViewport::new(&tape);
        let mut state = TapeViewportState::_new(width);

        let expected_buf =
            buf_for_tape_viewport(TapeEndcaps::None, margin, &vec![0; width]);

        render_tape(&mut term, widget, &mut state);

        assert_eq!(state.offset, width);
        backend.get().assert_buffer(&expected_buf);
    }

    #[test]
    fn shits_left_when_there_is_a_gap_after_the_tape() {
        let (mut term, backend) = terminal_for_tape(4);

        let tape = tape_from_script(">>>");
        let widget = TapeViewport::new(&tape);
        let mut state = TapeViewportState::_new(1);

        let expected_buf =
            buf_for_tape_viewport(TapeEndcaps::LeftRight, 3, &[0, 0, 0, 0]);

        render_tape(&mut term, widget, &mut state);

        assert_eq!(state.offset, 0);
        backend.get().assert_buffer(&expected_buf);
    }
}
