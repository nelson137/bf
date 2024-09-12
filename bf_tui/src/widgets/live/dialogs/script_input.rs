use std::cell::RefCell;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph, Wrap},
};
use tui_textarea::{CursorMove, TextArea};

use crate::{events::KeyEventExt, widgets::live::TextAreaExts};

use super::{
    button::{ButtonRowWidget, DialogButton},
    render_input, AppDialog, Dialog, DialogCommand, DialogFocus,
    DialogFocusController,
};

pub struct ScriptInputDialog<'textarea> {
    prompt: String,
    buttons: Vec<DialogButton>,
    focus: DialogFocusController,
    input: RefCell<TextArea<'textarea>>,
}

impl<'textarea> ScriptInputDialog<'textarea> {
    pub fn build() -> Dialog<'textarea> {
        let focus = DialogFocusController::new(vec![
            DialogFocus::Input,
            DialogFocus::button(0, DialogButton::Cancel),
            DialogFocus::button(1, DialogButton::Ok),
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
            prompt: "Input: ".to_string(),
            buttons,
            focus,
            input,
        };

        Dialog {
            title: " Input ",
            bg: Dialog::DEFAULT_BG,
            primary: Color::Green,
            fg: Dialog::DEFAULT_FG,
            dialog: Box::new(this),
        }
    }
}

impl AppDialog for ScriptInputDialog<'_> {
    fn on_event(&mut self, event: KeyEvent) -> super::DialogCommand {
        match event.code {
            KeyCode::Esc => DialogCommand::Dismissed,
            KeyCode::Char('c') if event.is_ctrl() => DialogCommand::Dismissed,

            KeyCode::Enter => {
                if self.focus.should_submit() {
                    DialogCommand::ScriptInputSubmitted(
                        self.input.borrow().to_string(),
                    )
                } else {
                    DialogCommand::Dismissed
                }
            }

            KeyCode::Tab => {
                self.focus.next();
                DialogCommand::None
            }

            KeyCode::BackTab => {
                self.focus.prev();
                DialogCommand::None
            }

            _ if self.focus.is_input() => {
                self.input.borrow_mut().on_event_single_line(event);
                DialogCommand::None
            }

            _ => DialogCommand::None,
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
            Dialog::DEFAULT_FG,
        )
        .render(buttons_area, buf);
    }
}
