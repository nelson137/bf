use std::iter;

use ratatui::{
    prelude::{Alignment, Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Copy, Clone)]
pub enum DialogueButton {
    Ok,
    Yes,
    Cancel,
}

pub const BUTTON_WIDTH: u16 = 10;

impl DialogueButton {
    pub const fn text(self) -> &'static str {
        /*
         * NOTE: Keep `BUTTON_WIDTH` up to date with the length of longest text.
         */
        match self {
            Self::Ok => "OK",
            Self::Yes => "YES",
            Self::Cancel => "CANCEL",
        }
    }

    pub const fn is_affirmative(self) -> bool {
        match self {
            Self::Ok | Self::Yes => true,
            Self::Cancel => false,
        }
    }
}

pub struct DialogueButtonWidget {
    kind: DialogueButton,
    fg: Color,
    selected: bool,
}

impl DialogueButtonWidget {
    pub const fn new(kind: DialogueButton, fg: Color, selected: bool) -> Self {
        Self { kind, fg, selected }
    }
}

impl Widget for DialogueButtonWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::new().style(if self.selected {
            Style::new()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Blue)
                .bg(self.fg)
        } else {
            Style::new().fg(self.fg)
        });

        Paragraph::new(self.kind.text())
            .block(block)
            .alignment(Alignment::Center)
            .render(area, buf);

        buf.get_mut(area.left(), area.y).set_char('[');
        buf.get_mut(area.right() - 1, area.y).set_char(']');
    }
}

pub struct ButtonRowWidget<'buttons> {
    buttons: &'buttons [DialogueButton],
    cursor: Option<u8>,
    fg: Color,
}

impl<'buttons> ButtonRowWidget<'buttons> {
    pub const fn new(
        buttons: &'buttons [DialogueButton],
        cursor: Option<u8>,
        fg: Color,
    ) -> Self {
        Self {
            buttons,
            cursor,
            fg,
        }
    }
}

impl Widget for ButtonRowWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut constraints = Vec::with_capacity(self.buttons.len() * 2 + 1);
        constraints.push(Constraint::Min(1));
        const BUTTON_AND_MARGIN: [Constraint; 2] =
            [Constraint::Length(BUTTON_WIDTH), Constraint::Length(2)];
        constraints.extend(
            iter::repeat(&BUTTON_AND_MARGIN)
                .take(self.buttons.len())
                .flatten(),
        );

        let layout = Layout::new()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(area);

        let buttons = self.buttons.iter().copied();
        let button_areas = layout.iter().copied().skip(1).step_by(2);
        for ((i, button), button_area) in buttons.enumerate().zip(button_areas)
        {
            let selected = matches!(self.cursor, Some(c) if c as usize == i);
            DialogueButtonWidget::new(button, self.fg, selected)
                .render(button_area, buf);
        }
    }
}
