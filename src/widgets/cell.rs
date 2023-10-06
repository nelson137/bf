use std::borrow::Cow;

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
