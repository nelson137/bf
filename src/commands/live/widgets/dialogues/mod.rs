use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{
        Block, BorderType, Borders, Clear, Padding, Paragraph, Widget, Wrap,
    },
};
use ratatui_textarea::{CursorMove, TextArea};

use crate::{
    commands::live::textarea::TextAreaExts, sublayouts, util::tui::KeyEventExt,
};

use self::{
    button::{ButtonRowWidget, DialogueButton},
    drop_shadow::DropShadowWidget,
};

mod button;
mod drop_shadow;

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let centered_vert_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area)[1];

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(centered_vert_area)[1]
}

enum DialogueKind<'textarea> {
    ConfirmUnsavedChanges(ButtonDialogueState),
    Error(ButtonDialogueState),
    FileSaveAs(PromptDialogueState<'textarea>),
    ScriptInput(PromptDialogueState<'textarea>),
    ScriptAutoInput(PromptDialogueState<'textarea>),
}

#[derive(Clone, PartialEq)]
enum DialogueAction {
    None,
    No,
    Yes,
    Submit(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogueCommand {
    None,
    Dismissed,
    ConfirmUnsavedChangesConfirmed,
    FileSaveAsSubmitted(String),
    ScriptInputSubmitted(String),
    ScriptAutoInputSubmitted(Option<u8>),
}

pub struct Dialogue<'textarea> {
    title: &'static str,
    bg: Color,
    primary: Color,
    fg: Color,
    kind: DialogueKind<'textarea>,
}

impl Dialogue<'_> {
    const DEFAULT_BG: Color = Color::Reset;
    const DEFAULT_FG: Color = Color::White;

    fn _prompt_str_input(value: Option<String>) -> TextArea<'static> {
        let mut input =
            TextArea::new(value.map(|v| vec![v]).unwrap_or_default());
        input.set_block(Block::new().borders(Borders::ALL));
        input.set_cursor_line_style(Style::new());
        input.move_cursor(CursorMove::End);
        input
    }

    pub fn confirm_unsaved_changes(message: impl Into<String>) -> Self {
        Self {
            title: " Confirm ",
            bg: Self::DEFAULT_BG,
            primary: Color::Yellow,
            fg: Self::DEFAULT_FG,
            kind: DialogueKind::ConfirmUnsavedChanges(ButtonDialogueState {
                message: message.into(),
                buttons: vec![DialogueButton::Cancel, DialogueButton::Yes],
                cursor: 0,
            }),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            title: " Error ",
            bg: Self::DEFAULT_BG,
            primary: Color::Red,
            fg: Self::DEFAULT_FG,
            kind: DialogueKind::Error(ButtonDialogueState {
                message: message.into(),
                buttons: vec![DialogueButton::Ok],
                cursor: 0,
            }),
        }
    }

    pub fn file_save_as(value: Option<impl Into<String>>) -> Self {
        Self {
            title: " Save As ",
            bg: Self::DEFAULT_BG,
            primary: Color::LightGreen,
            fg: Self::DEFAULT_FG,
            kind: DialogueKind::FileSaveAs(PromptDialogueState {
                prompt: "Filename: ".to_string(),
                buttons: vec![DialogueButton::Cancel, DialogueButton::Ok],
                cursor: PromptDialogueCursor::default(),
                input: Self::_prompt_str_input(value.map(Into::into)),
            }),
        }
    }

    pub fn script_input() -> Self {
        Self {
            title: " Input ",
            bg: Self::DEFAULT_BG,
            primary: Color::Green,
            fg: Self::DEFAULT_FG,
            kind: DialogueKind::ScriptInput(PromptDialogueState {
                prompt: "Input: ".to_string(),
                buttons: vec![DialogueButton::Cancel, DialogueButton::Ok],
                cursor: PromptDialogueCursor::default(),
                input: Self::_prompt_str_input(None),
            }),
        }
    }

    pub fn script_auto_input() -> Self {
        Self {
            title: " Auto-Input ",
            bg: Self::DEFAULT_BG,
            primary: Color::Green,
            fg: Self::DEFAULT_FG,
            kind: DialogueKind::ScriptAutoInput(PromptDialogueState {
                prompt: "Input (only the first byte will be used): "
                    .to_string(),
                buttons: vec![DialogueButton::Cancel, DialogueButton::Ok],
                cursor: PromptDialogueCursor::default(),
                input: Self::_prompt_str_input(None),
            }),
        }
    }
}

impl Dialogue<'_> {
    pub fn on_event(&mut self, event: KeyEvent) -> DialogueCommand {
        match &mut self.kind {
            DialogueKind::ConfirmUnsavedChanges(s) => {
                match s.on_event(event) {
                    DialogueAction::Yes => {
                        DialogueCommand::ConfirmUnsavedChangesConfirmed
                    }
                    DialogueAction::No => DialogueCommand::Dismissed,
                    _ => DialogueCommand::None,
                }
            }
            DialogueKind::Error(s) => match s.on_event(event) {
                DialogueAction::None => DialogueCommand::None,
                _ => DialogueCommand::Dismissed,
            },
            DialogueKind::FileSaveAs(s) => match s.on_event(event) {
                DialogueAction::Submit(value) => {
                    DialogueCommand::FileSaveAsSubmitted(value)
                }
                DialogueAction::No => DialogueCommand::Dismissed,
                _ => DialogueCommand::None,
            },
            DialogueKind::ScriptInput(s) => match s.on_event(event) {
                DialogueAction::Submit(value) => {
                    DialogueCommand::ScriptInputSubmitted(value)
                }
                DialogueAction::No => DialogueCommand::Dismissed,
                _ => DialogueCommand::None,
            },
            DialogueKind::ScriptAutoInput(s) => match s.on_event(event) {
                DialogueAction::Submit(value) => {
                    DialogueCommand::ScriptAutoInputSubmitted(
                        value.as_bytes().first().copied(),
                    )
                }
                DialogueAction::No => DialogueCommand::Dismissed,
                _ => DialogueCommand::None,
            },
        }
    }
}

impl<'textarea> Widget for &Dialogue<'textarea> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::new()
            .title(self.title)
            .title_style(Style::new().bg(self.bg).fg(self.primary))
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::new().bg(self.bg).fg(self.primary))
            .style(Style::new().bg(self.bg).fg(self.fg))
            .padding(Padding::uniform(1));
        let content_area = block.inner(area);
        block.render(area, buf);

        match &self.kind {
            DialogueKind::ConfirmUnsavedChanges(state)
            | DialogueKind::Error(state) => {
                self.render_button_dialogue(content_area, buf, state)
            }
            DialogueKind::FileSaveAs(state)
            | DialogueKind::ScriptInput(state)
            | DialogueKind::ScriptAutoInput(state) => {
                self.render_prompt_dialogue(content_area, buf, state)
            }
        }

        DropShadowWidget::new(2, 2).render(area, buf);
    }
}

impl Dialogue<'_> {
    fn render_button_dialogue(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &ButtonDialogueState,
    ) {
        let layout = Layout::new()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(0),    // Message
                Constraint::Length(1), // Space (skip)
                Constraint::Length(1), // Buttons
            ])
            .split(area);
        sublayouts!([text_area, _, buttons_area] = layout);

        // Message

        Paragraph::new(&*state.message)
            .wrap(Wrap { trim: false })
            .render(text_area, buf);

        // Buttons

        ButtonRowWidget::new(&state.buttons, Some(state.cursor), self.fg)
            .render(buttons_area, buf);
    }

    fn render_prompt_dialogue(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &PromptDialogueState,
    ) {
        let layout = Layout::new()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1), // Prompt
                Constraint::Length(1), // Space (skip)
                Constraint::Length(3), // Input
                Constraint::Min(0),    // Space (skip)
                Constraint::Length(1), // Buttons
            ])
            .split(area);
        sublayouts!([prompt_area, _, input_area, _, buttons_area] = layout);

        // Prompt

        Paragraph::new(&*state.prompt)
            .wrap(Wrap { trim: false })
            .render(prompt_area, buf);

        // Input

        state.input.widget().render(input_area, buf);

        // Buttons

        let cursor = match state.cursor {
            PromptDialogueCursor::Input => None,
            PromptDialogueCursor::Button(index) => Some(index),
        };
        ButtonRowWidget::new(&state.buttons, cursor, self.fg)
            .render(buttons_area, buf);
    }
}

trait DialogueState {
    fn on_event(&mut self, event: KeyEvent) -> DialogueAction;
}

struct ButtonDialogueState {
    message: String,
    buttons: Vec<DialogueButton>,
    cursor: u8,
}

impl ButtonDialogueState {
    fn focus_next_button(&mut self) {
        let len = self.buttons.len() as u8;
        self.cursor = (self.cursor + 1) % len;
    }

    fn focus_prev_button(&mut self) {
        let len = self.buttons.len() as u8;
        self.cursor = (self.cursor + len - 1) % len;
    }
}

impl DialogueState for ButtonDialogueState {
    fn on_event(&mut self, event: KeyEvent) -> DialogueAction {
        match event.code {
            KeyCode::Esc => DialogueAction::No,
            KeyCode::Char('c') if event.is_ctrl() => DialogueAction::No,

            KeyCode::Enter => {
                if self.cursor == 0 {
                    DialogueAction::No
                } else {
                    DialogueAction::Yes
                }
            }

            KeyCode::Tab | KeyCode::Right => {
                self.focus_next_button();
                DialogueAction::None
            }

            KeyCode::BackTab | KeyCode::Left => {
                self.focus_prev_button();
                DialogueAction::None
            }

            _ => DialogueAction::None,
        }
    }
}

struct PromptDialogueState<'textarea> {
    prompt: String,
    buttons: Vec<DialogueButton>,
    cursor: PromptDialogueCursor,
    input: TextArea<'textarea>,
}

impl PromptDialogueState<'_> {
    fn cursor_should_submit(&self) -> bool {
        match self.cursor {
            PromptDialogueCursor::Input => true,
            PromptDialogueCursor::Button(index) => {
                self.buttons[index as usize].is_affirmative()
            }
        }
    }

    fn focus_next(&mut self) {
        self.cursor = match self.cursor {
            PromptDialogueCursor::Input => PromptDialogueCursor::Button(0),
            PromptDialogueCursor::Button(index) => {
                let next = index + 1;
                if next >= self.buttons.len() as u8 {
                    PromptDialogueCursor::Input
                } else {
                    PromptDialogueCursor::Button(next)
                }
            }
        };
    }

    fn focus_prev(&mut self) {
        self.cursor = match self.cursor {
            PromptDialogueCursor::Input => {
                PromptDialogueCursor::Button(self.buttons.len() as u8 - 1)
            }
            PromptDialogueCursor::Button(index) => {
                match index.checked_sub(1) {
                    None => PromptDialogueCursor::Input,
                    Some(next) => PromptDialogueCursor::Button(next),
                }
            }
        }
    }
}

impl DialogueState for PromptDialogueState<'_> {
    fn on_event(&mut self, event: KeyEvent) -> DialogueAction {
        match event.code {
            KeyCode::Esc => DialogueAction::No,
            KeyCode::Char('c') if event.is_ctrl() => DialogueAction::No,

            KeyCode::Enter => {
                if self.cursor_should_submit() {
                    DialogueAction::Submit(self.input.to_string())
                } else {
                    DialogueAction::No
                }
            }

            KeyCode::Tab => {
                self.focus_next();
                DialogueAction::None
            }

            KeyCode::BackTab => {
                self.focus_prev();
                DialogueAction::None
            }

            _ if self.cursor == PromptDialogueCursor::Input => {
                self.input.on_event_single_line(event);
                DialogueAction::None
            }

            _ => DialogueAction::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum PromptDialogueCursor {
    #[default]
    Input,
    Button(u8),
}
