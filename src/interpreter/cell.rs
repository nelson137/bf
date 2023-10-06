use std::{borrow::Cow, num::Wrapping};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::util::tui::{
    sublayouts, TapeBorderHorizontal, TAPE_BORDER_SET,
    TAPE_HORIZONTAL_BORDER_BOTTOM, TAPE_HORIZONTAL_BORDER_TOP,
};

#[derive(Debug, Clone)]
pub struct Cell(Wrapping<u8>);

impl Cell {
    pub fn new() -> Self {
        Self(Wrapping(0))
    }

    pub fn inc(&mut self) {
        self.0 += Wrapping(1);
    }

    pub fn dec(&mut self) {
        self.0 -= Wrapping(1);
    }

    pub fn value(&self) -> u8 {
        (self.0).0
    }

    pub fn set(&mut self, value: u8) {
        self.0 = Wrapping(value);
    }
}

pub struct CellDisplay {
    pub value: u8,
    pub left_cap: bool,
    pub right_border_cap: Option<bool>,
    pub is_highlighted: bool,
    pub ascii: bool,
}

impl CellDisplay {
    pub fn is_highlighted(&self) -> bool {
        self.is_highlighted
    }

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
        macro_rules! fmt {
            ($value:expr) => {
                Cow::Owned(format!("{:^3}", $value))
            };
        }
        if self.ascii {
            let c = self.value as char;
            match c {
                '\0' => Cow::Borrowed(r"\0 "),
                ' ' => Cow::Borrowed("' '"),
                '\t' | '\n' | '\r' | '!'..='~' => fmt!(c.escape_default()),
                _ => fmt!(c as u8),
            }
        } else {
            fmt!(self.value)
        }
    }
}

impl Widget for CellDisplay {
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

        let border = Span::raw(TAPE_BORDER_SET.vertical);
        let display_value = self.display_value();
        let value = if self.is_highlighted {
            display_value.add_modifier(Modifier::REVERSED)
        } else {
            Span::raw(display_value)
        };
        Paragraph::new(Line::from(vec![
            border.clone(),
            value,
            border.clone(),
        ]))
        .render(middle_area, buf);

        Paragraph::new(self.display_bottom()).render(bottom_area, buf);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_initializes_to_0() {
        let cell = Cell::new();
        assert_eq!(cell.0 .0, 0);
    }

    #[test]
    fn inc_increases_value_by_1() {
        let value = fastrand::u8(..u8::MAX);
        let mut cell = Cell(Wrapping(value));
        cell.inc();
        assert_eq!(cell.0 .0, value + 1);
    }

    #[test]
    fn dec_decreases_value_by_1() {
        let value = fastrand::u8(1..);
        let mut cell = Cell(Wrapping(value));
        cell.dec();
        assert_eq!(cell.0 .0, value - 1);
    }

    #[test]
    fn value_returns_the_value() {
        let value = fastrand::u8(..);
        let cell = Cell(Wrapping(value));
        assert_eq!(cell.value(), value);
    }

    #[test]
    fn set_updates_the_value() {
        let value = fastrand::u8(..);
        let mut cell = Cell::new();
        cell.set(value);
        assert_eq!(cell.0 .0, value);
    }
}
