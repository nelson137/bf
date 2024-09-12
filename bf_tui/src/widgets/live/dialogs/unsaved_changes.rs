use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

use crate::events::KeyEventExt;

use super::{
    button::{ButtonRowWidget, DialogButton},
    AppDialog, Dialog, DialogCommand, DialogFocus, DialogFocusController,
};

pub struct UnsavedChangesDialog {
    message: String,
    buttons: Vec<DialogButton>,
    focus: DialogFocusController,
}

impl UnsavedChangesDialog {
    pub fn build() -> Dialog<'static> {
        let message = "Warning:\n\n\
            There are unsaved changes, are you sure you want to quit?";

        let focus = DialogFocusController::new(vec![
            DialogFocus::button(0, DialogButton::Cancel),
            DialogFocus::button(1, DialogButton::Yes),
        ]);

        let buttons = focus.to_buttons();

        let this = Self {
            message: message.to_string(),
            buttons,
            focus,
        };

        Dialog {
            title: " Confirm ",
            bg: Dialog::DEFAULT_BG,
            primary: Color::Yellow,
            fg: Dialog::DEFAULT_FG,
            dialog: Box::new(this),
        }
    }
}

impl AppDialog for UnsavedChangesDialog {
    fn on_event(&mut self, event: KeyEvent) -> super::DialogCommand {
        match event.code {
            KeyCode::Esc => DialogCommand::Dismissed,
            KeyCode::Char('c') if event.is_ctrl() => DialogCommand::Dismissed,

            KeyCode::Enter => {
                if self.focus.should_submit() {
                    DialogCommand::ConfirmUnsavedChangesConfirmed
                } else {
                    DialogCommand::Dismissed
                }
            }

            KeyCode::Tab | KeyCode::Right => {
                self.focus.next();
                DialogCommand::None
            }

            KeyCode::BackTab | KeyCode::Left => {
                self.focus.prev();
                DialogCommand::None
            }

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

        ButtonRowWidget::new(
            &self.buttons,
            self.focus.button_cursor(),
            Dialog::DEFAULT_FG,
        )
        .render(buttons_area, buf);
    }
}
