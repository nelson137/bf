use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

use crate::events::KeyEventExt;

use super::{
    button::{ButtonRowWidget, DialogButton},
    AppDialog, Dialog, DialogCommand,
};

pub struct ErrorDialog {
    message: String,
    buttons: Vec<DialogButton>,
}

impl ErrorDialog {
    pub fn build(message: String) -> Dialog<'static> {
        let this = Self {
            message,
            buttons: vec![DialogButton::Ok],
        };

        Dialog {
            title: " Error ",
            bg: Dialog::DEFAULT_BG,
            primary: Color::Red,
            fg: Dialog::DEFAULT_FG,
            dialog: Box::new(this),
        }
    }
}

impl AppDialog for ErrorDialog {
    fn on_event(&mut self, event: KeyEvent) -> super::DialogCommand {
        match event.code {
            KeyCode::Esc => DialogCommand::Dismissed,
            KeyCode::Char('c') if event.is_ctrl() => DialogCommand::Dismissed,

            KeyCode::Enter => DialogCommand::Dismissed,

            _ => DialogCommand::None,
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

        ButtonRowWidget::new(&self.buttons, Some(0), Dialog::DEFAULT_FG)
            .render(buttons_area, buf);
    }
}
