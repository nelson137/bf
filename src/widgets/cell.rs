use std::borrow::Cow;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::util::tui::{
    sublayouts, TapeBorderHorizontal, TAPE_BORDER_SET,
    TAPE_HORIZONTAL_BORDER_BOTTOM, TAPE_HORIZONTAL_BORDER_TOP,
};

#[derive(Default)]
pub struct CellWidget {
    pub value: u8,
    pub left_cap: bool,
    pub right_border_cap: Option<bool>,
    pub is_highlighted: bool,
    pub ascii: bool,
}

impl CellWidget {
    fn display_horizontal_edge(&self, edge: TapeBorderHorizontal) -> String {
        String::with_capacity(5)
            + edge.left(self.left_cap)
            + &edge.middle().repeat(3)
            + self.right_border_cap.map(|c| edge.right(c)).unwrap_or("")
    }

    pub fn display_top(&self) -> String {
        self.display_horizontal_edge(TAPE_HORIZONTAL_BORDER_TOP)
    }

    pub fn display_bottom(&self) -> String {
        self.display_horizontal_edge(TAPE_HORIZONTAL_BORDER_BOTTOM)
    }

    pub fn display_value(&self) -> Cow<str> {
        macro_rules! owned {
            ($value:expr) => {
                Cow::Owned(format!("{:^3}", $value))
            };
        }
        if self.ascii {
            let c = self.value as char;
            match c {
                '\0' => Cow::Borrowed(r"\0 "),
                '\t' => Cow::Borrowed(r"\t "),
                '\r' => Cow::Borrowed(r"\r "),
                '\n' => Cow::Borrowed(r"\n "),
                ' ' => Cow::Borrowed("' '"),
                '!'..='~' => owned!(c),
                _ => owned!(c as u8),
            }
        } else {
            owned!(self.value)
        }
    }
}

impl Widget for CellWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);
        sublayouts!([top_area, middle_area, bottom_area] = layout);

        Paragraph::new(self.display_top()).render(top_area, buf);

        let left_border = Span::raw(TAPE_BORDER_SET.vertical);
        let right_border = Span::raw(match self.right_border_cap {
            Some(_) => TAPE_BORDER_SET.vertical,
            None => "",
        });
        let display_value = self.display_value();
        let value = if self.is_highlighted {
            display_value.reversed()
        } else {
            Span::from(display_value)
        };
        Paragraph::new(Line::from(vec![left_border, value, right_border]))
            .render(middle_area, buf);

        Paragraph::new(self.display_bottom()).render(bottom_area, buf);
    }
}

#[cfg(test)]
mod test {
    use test_case::test_case;

    use super::*;

    mod util {
        use ratatui::{style::Style, Terminal};

        use crate::util::test::{terminal, MyTestBackend};

        use super::super::*;

        pub fn num_cell_with_value(value: u8) -> CellWidget {
            CellWidget {
                value,
                ..Default::default()
            }
        }

        pub fn ascii_cell_with_value(value: u8) -> CellWidget {
            CellWidget {
                ascii: true,
                value,
                ..Default::default()
            }
        }

        pub fn terminal_for_cell() -> (Terminal<MyTestBackend>, MyTestBackend)
        {
            terminal(5, 3)
        }

        pub fn buf_for_cell(
            left_cap: bool,
            right: Option<bool>,
            highlighted: bool,
        ) -> Buffer {
            let top_left = if left_cap { "┌" } else { "┬" };
            let top_right_char =
                |capped: bool| if capped { "┐" } else { "┬" };
            let top_right = right.map(top_right_char).unwrap_or(" ");
            let top = String::with_capacity(5) + top_left + "───" + top_right;

            let middle = String::with_capacity(5)
                + "│ 0 "
                + right.map(|_| "│").unwrap_or(" ");

            let bottom_left = if left_cap { "└" } else { "┴" };
            let bottom_right_char =
                |capped: bool| if capped { "┘" } else { "┴" };
            let bottom_right = right.map(bottom_right_char).unwrap_or(" ");
            let bottom =
                String::with_capacity(5) + bottom_left + "───" + bottom_right;

            let mut buffer = Buffer::with_lines(vec![top, middle, bottom]);

            if highlighted {
                buffer.set_string(1, 1, " 0 ", Style::default().reversed());
            }

            buffer
        }

        pub fn render_cell(
            term: &mut Terminal<MyTestBackend>,
            widget: CellWidget,
        ) {
            term.draw(|f| f.render_widget(widget, f.size())).unwrap();
        }
    }
    use util::*;

    #[test_case(4, " 4 " ; "4")]
    #[test_case(42, "42 " ; "42")]
    #[test_case(255, "255" ; "255")]
    fn display_numerical_value(value: u8, expected_str: &str) {
        let widget = num_cell_with_value(value);
        assert_eq!(widget.display_value(), expected_str);
    }

    #[test_case(b'\0', r"\0 " ; "null")]
    #[test_case(b'\t', r"\t " ; "tab")]
    #[test_case(b'\r', r"\r " ; "carriage return")]
    #[test_case(b'\n', r"\n " ; "newline")]
    #[test_case(b' ', "' '" ; "space")]
    #[test_case(b'A', " A " ; "capital a")]
    #[test_case(127, "127" ; "127")]
    fn display_ascii_value(value: u8, expected_str: &str) {
        let widget = ascii_cell_with_value(value);
        assert_eq!(widget.display_value(), expected_str);
    }

    #[test]
    fn render_left_cap() {
        let (mut term, backend) = terminal_for_cell();

        let widget = CellWidget {
            left_cap: true,
            ..Default::default()
        };

        let expected_buf = buf_for_cell(true, None, false);

        render_cell(&mut term, widget);

        backend.get().assert_buffer(&expected_buf);
    }

    #[test_case(None ; "no cap")]
    #[test_case(Some(false) ; "uncapped")]
    #[test_case(Some(true) ; "capped")]
    fn render_right(right_border_cap: Option<bool>) {
        let (mut term, backend) = terminal_for_cell();

        let widget = CellWidget {
            right_border_cap,
            ..Default::default()
        };

        let expected_buf = buf_for_cell(false, right_border_cap, false);

        render_cell(&mut term, widget);

        backend.get().assert_buffer(&expected_buf);
    }

    #[test]
    fn render_highlight() {
        let (mut term, backend) = terminal_for_cell();

        let widget = CellWidget {
            is_highlighted: true,
            ..Default::default()
        };

        let expected_buf = buf_for_cell(false, None, true);

        render_cell(&mut term, widget);

        backend.get().assert_buffer(&expected_buf);
    }
}
