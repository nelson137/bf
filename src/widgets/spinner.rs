use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{Paragraph, Widget},
};

const SPINNER: [&str; 4] = ["│", "╱", "─", "╲"];

#[derive(Clone, Copy, Default)]
pub struct Spinner(usize);

impl Spinner {
    pub fn tick(&mut self) {
        self.0 = (self.0 + 1) % SPINNER.len();
    }
}

impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(SPINNER[self.0]).render(area, buf);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod util {
        use ratatui::{
            backend::{Backend, TestBackend},
            Terminal,
        };

        use crate::util::test::terminal;

        use super::super::*;

        pub fn terminal_for_cell() -> Terminal<TestBackend> {
            terminal(1, 1)
        }

        pub fn render_cell(
            term: &mut Terminal<impl Backend>,
            widget: Spinner,
        ) {
            term.draw(|f| f.render_widget(widget, f.size())).unwrap();
        }
    }
    use util::*;

    #[test]
    fn tick_increases_index_by_1() {
        let index = fastrand::usize(..(SPINNER.len() - 1));
        let mut spinner = Spinner(index);
        spinner.tick();
        assert_eq!(spinner.0, index + 1);
    }

    #[test]
    fn tick_wraps_to_0() {
        let mut spinner = Spinner(SPINNER.len() - 1);
        spinner.tick();
        assert_eq!(spinner.0, 0);
    }

    #[test]
    fn renders_a_frame() {
        let mut term = terminal_for_cell();

        let index = fastrand::usize(0..SPINNER.len());
        let expected_buf = Buffer::with_lines(vec![SPINNER[index]]);
        let spinner = Spinner(index);

        render_cell(&mut term, spinner);

        term.backend().assert_buffer(&expected_buf);
    }
}
