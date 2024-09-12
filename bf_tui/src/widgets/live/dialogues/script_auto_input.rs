use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph, Wrap},
};
use tui_textarea::{CursorMove, TextArea};

use crate::{events::KeyEventExt, widgets::live::TextAreaExts};

use super::{
    button::{ButtonRowWidget, DialogueButton},
    render_input, AppDialogue, Dialogue, DialogueCommand, DialogueFocus,
    DialogueFocusController,
};

pub struct ScriptAutoInputDialogue<'textarea> {
    prompt: String,
    buttons: Vec<DialogueButton>,
    focus: DialogueFocusController,
    input: RefCell<TextArea<'textarea>>,
}

impl<'textarea> ScriptAutoInputDialogue<'textarea> {
    pub fn build() -> Dialogue<'textarea> {
        let focus = DialogueFocusController::new(vec![
            DialogueFocus::Input,
            DialogueFocus::button(0, DialogueButton::Cancel),
            DialogueFocus::button(1, DialogueButton::Ok),
        ]);

        let buttons = focus.to_buttons();

        let input = {
            let mut input = tui_textarea::TextArea::new(vec![]);
            input.set_block(Block::bordered());
            input.set_cursor_line_style(Style::new());
            input.move_cursor(CursorMove::End);
            RefCell::new(input)
        };

        let this = Self {
            prompt: "Input (only the first byte will be used): ".to_string(),
            buttons,
            focus,
            input,
        };

        Dialogue {
            title: " Auto-Input ",
            bg: Dialogue::DEFAULT_BG,
            primary: Color::Green,
            fg: Dialogue::DEFAULT_FG,
            dialogue: Box::new(this),
        }
    }
}

impl AppDialogue for ScriptAutoInputDialogue<'_> {
    fn on_event(&mut self, event: KeyEvent) -> super::DialogueCommand {
        match event.code {
            KeyCode::Esc => DialogueCommand::Dismissed,
            KeyCode::Char('c') if event.is_ctrl() => {
                DialogueCommand::Dismissed
            }

            KeyCode::Enter => {
                if self.focus.should_submit() {
                    DialogueCommand::ScriptAutoInputSubmitted(
                        self.input
                            .borrow()
                            .lines()
                            .first()
                            .and_then(|l| l.as_bytes().first().copied()),
                    )
                } else {
                    DialogueCommand::Dismissed
                }
            }

            KeyCode::Tab => {
                self.focus.next();
                DialogueCommand::None
            }

            KeyCode::BackTab => {
                self.focus.prev();
                DialogueCommand::None
            }

            _ if self.focus.is_input() => {
                self.input.borrow_mut().on_event_single_line(event);
                DialogueCommand::None
            }

            _ => DialogueCommand::None,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical(vec![
            Constraint::Length(1), // Prompt
            Constraint::Length(1), // Space (skip)
            Constraint::Length(3), // Input
            Constraint::Fill(1),   // Space (skip)
            Constraint::Length(1), // Buttons
        ])
        .split(area);
        sublayouts!([prompt_area, _, input_area, _, buttons_area] = layout);

        // Prompt

        Paragraph::new(&*self.prompt)
            .wrap(Wrap { trim: false })
            .render(prompt_area, buf);

        // Input

        let mut input = self.input.borrow_mut();
        render_input(&mut input, input_area, buf, self.focus.is_input());

        // Buttons

        ButtonRowWidget::new(
            &self.buttons,
            self.focus.button_cursor(),
            Dialogue::DEFAULT_FG,
        )
        .render(buttons_area, buf);
    }
}
