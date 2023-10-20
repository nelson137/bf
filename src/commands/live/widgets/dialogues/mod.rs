use std::iter;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};
use ratatui_textarea::{CursorMove, TextArea};

use crate::{
    commands::live::{
        textarea::TextAreaExts, widgets::dialogues::button::BUTTON_WIDTH,
    },
    sublayouts,
    util::tui::KeyEventExt,
};

use self::{
    button::{DialogueButton, DialogueButtonWidget},
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
                button_cursor: 0,
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
                button_cursor: 0,
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
                button_state: ButtonDialogueState {
                    message: "Filename: ".to_string(),
                    buttons: vec![DialogueButton::Cancel, DialogueButton::Ok],
                    button_cursor: 0,
                },
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
                button_state: ButtonDialogueState {
                    message: "Input: ".to_string(),
                    buttons: vec![DialogueButton::Cancel, DialogueButton::Ok],
                    button_cursor: 0,
                },
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
                button_state: ButtonDialogueState {
                    message: "Input (only the first byte will be used): "
                        .to_string(),
                    buttons: vec![DialogueButton::Cancel, DialogueButton::Ok],
                    button_cursor: 0,
                },
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
            .style(Style::new().bg(self.bg).fg(self.fg));
        let content_area = block.inner(area).inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });
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
        sublayouts!([text_area, _, all_buttons_area] = layout);

        // Message

        Paragraph::new(&*state.message)
            .wrap(Wrap { trim: false })
            .render(text_area, buf);

        // Buttons

        let mut constraints = Vec::with_capacity(state.buttons.len() * 2 + 1);
        constraints.push(Constraint::Min(1));
        const BUTTON_AND_MARGIN: [Constraint; 2] =
            [Constraint::Length(BUTTON_WIDTH), Constraint::Length(2)];
        constraints.extend(
            iter::repeat(&BUTTON_AND_MARGIN)
                .take(state.buttons.len())
                .flatten(),
        );

        let layout = Layout::new()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(all_buttons_area);

        let buttons = state.buttons.iter().copied();
        let button_areas = layout.iter().copied().skip(1).step_by(2);
        for ((i, button), button_area) in buttons.enumerate().zip(button_areas)
        {
            let selected = state.button_cursor as usize == i;
            DialogueButtonWidget::new(button, self.fg, selected)
                .render(button_area, buf);
        }
    }

    fn render_prompt_dialogue(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &PromptDialogueState,
    ) {
        self.render_button_dialogue(area, buf, &state.button_state);

        let layout = Layout::new()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
            .split(area.inner(&Margin {
                horizontal: 0,
                vertical: 2,
            }));
        sublayouts!([input_area, _] = layout);

        state.input.widget().render(input_area, buf);
    }
}

trait DialogueState {
    fn on_event(&mut self, event: KeyEvent) -> DialogueAction;
}

struct ButtonDialogueState {
    message: String,
    buttons: Vec<DialogueButton>,
    button_cursor: u8,
}

impl ButtonDialogueState {
    fn focus_next_button(&mut self) {
        let len = self.buttons.len() as u8;
        self.button_cursor = (self.button_cursor + 1) % len;
    }

    fn focus_prev_button(&mut self) {
        let len = self.buttons.len() as u8;
        self.button_cursor = (self.button_cursor + len - 1) % len;
    }
}

impl DialogueState for ButtonDialogueState {
    fn on_event(&mut self, event: KeyEvent) -> DialogueAction {
        match event.code {
            KeyCode::Esc => DialogueAction::No,
            KeyCode::Char('c') if event.is_ctrl() => DialogueAction::No,

            KeyCode::Enter => {
                if self.button_cursor == 0 {
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

// TODO: refactor to not have a `ButtonDialogueState`
struct PromptDialogueState<'textarea> {
    button_state: ButtonDialogueState,
    input: TextArea<'textarea>,
}

impl DialogueState for PromptDialogueState<'_> {
    fn on_event(&mut self, event: KeyEvent) -> DialogueAction {
        match self.button_state.on_event(event) {
            DialogueAction::None => {
                self.input.on_event_single_line(event);
                DialogueAction::None
            }
            DialogueAction::Yes => {
                DialogueAction::Submit(self.input.to_string())
            }
            _ => DialogueAction::No,
        }
    }
}
