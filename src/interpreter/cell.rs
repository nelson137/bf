use std::num::Wrapping;

use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Paragraph, Widget},
};

use crate::util::{
    common::StrExt,
    tui::{TAPE_BORDER_SET, LineSymbolsExt, TapeBorderHorizontal},
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

    pub fn set(&mut self, value: char) {
        self.0 = Wrapping(value as u8);
    }

}

pub struct CellDisplay<'a> {
    cell: &'a Cell,
    left_cap: bool,
    right_border_cap: Option<bool>,
    is_highlighted: bool,
    ascii: bool,
}

impl<'a> CellDisplay<'a> {

    pub fn new(
        cell: &'a Cell,
        left_cap: bool,
        right_border_cap: Option<bool>,
        is_highlighted: bool,
        ascii: bool,
    ) -> Self {
        Self {
            cell,
            left_cap,
            right_border_cap,
            is_highlighted,
            ascii,
        }
    }

    pub fn is_highlighted(&self) -> bool {
        self.is_highlighted
    }

    fn display_horizontal_edge(&self, edge: TapeBorderHorizontal) -> String {
        let mut buf = String::with_capacity(5);

        buf.push_str(edge.left(self.left_cap));

        buf.push_str(&edge.middle().repeated(3));

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

    pub fn display_value(&self) -> String {
        let num = self.cell.value().to_string();
        let c = self.cell.ascii();
        let escaped = c.escape_default().to_string();
        let value_str = match c {
            _ if !self.ascii => &num,
            '\0' => r"\0",
            ' ' => "' '",
            '\t' | '\n' | '\r' | '!'..='~' => &escaped,
            _ => &num,
        };
        format!("{:^3}", value_str)
    }

}

impl<'a> Widget for CellDisplay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let line_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);

        Paragraph::new(self.display_top())
            .render(line_areas[0], buf);

        let border = Span::raw(TAPE_BORDER_SET.vertical);
        let style = match self.is_highlighted {
            true => Style::default().bg(Color::White).fg(Color::Black),
            false => Style::default(),
        };
        let value = Span::styled(self.display_value(), style);
        Paragraph::new(Spans::from(
            vec![border.clone(), value, border.clone()]
        )).render(line_areas[1], buf);

        Paragraph::new(self.display_bottom())
            .render(line_areas[2], buf);
    }
}
