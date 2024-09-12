use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

use crate::events::KeyEventExt;

use super::{
    button::{ButtonRowWidget, DialogueButton},
    AppDialogue, Dialogue, DialogueCommand,
};

pub struct ErrorDialogue {
    message: String,
    buttons: Vec<DialogueButton>,
}

impl ErrorDialogue {
    pub fn build(message: String) -> Dialogue<'static> {
        let this = Self {
            message,
            buttons: vec![DialogueButton::Ok],
        };

        Dialogue {
            title: " Error ",
            bg: Dialogue::DEFAULT_BG,
            primary: Color::Red,
            fg: Dialogue::DEFAULT_FG,
            dialogue: Box::new(this),
        }
    }
}

impl AppDialogue for ErrorDialogue {
    fn on_event(&mut self, event: KeyEvent) -> super::DialogueCommand {
        match event.code {
            KeyCode::Esc => DialogueCommand::Dismissed,
            KeyCode::Char('c') if event.is_ctrl() => {
                DialogueCommand::Dismissed
            }

            KeyCode::Enter => DialogueCommand::Dismissed,

            _ => DialogueCommand::None,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical(vec![
            Constraint::Fill(1),   // Message
            Constraint::Length(1), // Buttons
        ])
        .spacing(1)
        .split(area);
        sublayouts!([text_area, buttons_area] = layout);

        // Message

        Paragraph::new(&*self.message)
            .wrap(Wrap { trim: false })
            .render(text_area, buf);

        // Buttons

        ButtonRowWidget::new(&self.buttons, Some(0), Dialogue::DEFAULT_FG)
            .render(buttons_area, buf);
    }
}
