use ratatui::{
    prelude::{Alignment, Buffer, Rect},
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
    pub fn text(self) -> &'static str {
        /*
         * NOTE: Keep `BUTTON_WIDTH` up to date with the length of longest text.
         */
        match self {
            Self::Ok => "OK",
            Self::Yes => "YES",
            Self::Cancel => "CANCEL",
        }
    }
}

pub struct DialogueButtonWidget {
    kind: DialogueButton,
    fg: Color,
    selected: bool,
}

impl DialogueButtonWidget {
    pub fn new(kind: DialogueButton, fg: Color, selected: bool) -> Self {
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
