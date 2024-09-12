use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

use crate::events::KeyEventExt;

use super::{
    button::{ButtonRowWidget, DialogueButton},
    AppDialogue, Dialogue, DialogueCommand, DialogueFocus,
    DialogueFocusController,
};

pub struct UnsavedChangesDialogue {
    message: String,
    buttons: Vec<DialogueButton>,
    focus: DialogueFocusController,
}

impl UnsavedChangesDialogue {
    pub fn build() -> Dialogue<'static> {
        let message = "Warning:\n\n\
            There are unsaved changes, are you sure you want to quit?";

        let focus = DialogueFocusController::new(vec![
            DialogueFocus::button(0, DialogueButton::Cancel),
            DialogueFocus::button(1, DialogueButton::Yes),
        ]);

        let buttons = focus.to_buttons();

        let this = Self {
            message: message.to_string(),
            buttons,
            focus,
        };

        Dialogue {
            title: " Confirm ",
            bg: Dialogue::DEFAULT_BG,
            primary: Color::Yellow,
            fg: Dialogue::DEFAULT_FG,
            dialogue: Box::new(this),
        }
    }
}

impl AppDialogue for UnsavedChangesDialogue {
    fn on_event(&mut self, event: KeyEvent) -> super::DialogueCommand {
        match event.code {
            KeyCode::Esc => DialogueCommand::Dismissed,
            KeyCode::Char('c') if event.is_ctrl() => {
                DialogueCommand::Dismissed
            }

            KeyCode::Enter => {
                if self.focus.should_submit() {
                    DialogueCommand::ConfirmUnsavedChangesConfirmed
                } else {
                    DialogueCommand::Dismissed
                }
            }

            KeyCode::Tab | KeyCode::Right => {
                self.focus.next();
                DialogueCommand::None
            }

            KeyCode::BackTab | KeyCode::Left => {
                self.focus.prev();
                DialogueCommand::None
            }

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

        ButtonRowWidget::new(
            &self.buttons,
            self.focus.button_cursor(),
            Dialogue::DEFAULT_FG,
        )
        .render(buttons_area, buf);
    }
}
