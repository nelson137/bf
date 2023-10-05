use std::{borrow::Cow, num::Wrapping};

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::util::tui::{
    sublayouts, LineSymbolsExt, TapeBorderHorizontal, TAPE_BORDER_SET,
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

    pub fn ascii(&self) -> char {
        self.value() as char
    }

    pub fn set(&mut self, value: u8) {
        self.0 = Wrapping(value);
    }
}

pub struct CellDisplay<'a> {
    pub cell: &'a Cell,
    pub left_cap: bool,
    pub right_border_cap: Option<bool>,
    pub is_highlighted: bool,
    pub ascii: bool,
}

impl<'a> CellDisplay<'a> {
    pub fn is_highlighted(&self) -> bool {
        self.is_highlighted
    }

    fn display_horizontal_edge(&self, edge: TapeBorderHorizontal) -> String {
        let mut buf = String::with_capacity(5);

        buf.push_str(edge.left(self.left_cap));

        buf.push_str(&edge.middle().repeat(3));

        if let Some(right_cap) = self.right_border_cap {
            buf.push_str(edge.right(right_cap));
        }

        buf
    }

    pub fn display_top(&self) -> String {
        self.display_horizontal_edge(TAPE_BORDER_SET.top())
    }

    pub fn display_bottom(&self) -> String {
        self.display_horizontal_edge(TAPE_BORDER_SET.bottom())
    }

    pub fn display_value(&self) -> Cow<str> {
        macro_rules! fmt {
            ($value:expr) => {
                Cow::Owned(format!("{:^3}", $value))
            };
        }
        if self.ascii {
            let c = self.cell.ascii();
            match c {
                '\0' => Cow::Borrowed(r"\0 "),
                ' ' => Cow::Borrowed("' '"),
                '\t' | '\n' | '\r' | '!'..='~' => fmt!(c.escape_default()),
                _ => fmt!(c as u8),
            }
        } else {
            fmt!(self.cell.value())
        }
    }
}

impl Widget for CellDisplay<'_> {
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
