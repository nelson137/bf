use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::{
        Alignment, Buffer, Constraint, Direction, Layout, Margin, Rect,
    },
    style::{Color, Style, Styled, Stylize},
    text::Text,
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget, Wrap},
};
use ratatui_textarea::TextArea;

use crate::{
    commands::live::textarea::TextAreaExts, sublayouts, util::tui::KeyEventExt,
};

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
    fg: Color,
    kind: DialogueKind<'textarea>,
}

impl Dialogue<'_> {
    fn _prompt_str_input(value: Option<String>) -> TextArea<'static> {
        let mut input =
            TextArea::new(value.map(|v| vec![v]).unwrap_or_default());
        input.set_block(Block::new().borders(Borders::ALL));
        input
    }
}

impl Dialogue<'_> {
    pub fn confirm_unsaved_changes(message: impl Into<String>) -> Self {
        Self {
            title: " Confirm ",
            bg: Color::Black,
            fg: Color::Yellow,
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
            bg: Color::Black,
            fg: Color::Red,
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
            bg: Color::Black,
            fg: Color::Green,
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
            bg: Color::Black,
            fg: Color::Green,
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
            bg: Color::Black,
            fg: Color::Green,
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

        // Margin box (single column on left and right)
        let block = Block::new()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::new().fg(self.bg).bg(self.bg));
        let dialogue_area = block.inner(area);
        block.render(area, buf);

        // Box with outline and title
        // TODO: get rid of margin box, just render this block in `area`
        let block = Block::new()
            .title(self.title)
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .border_style(Style::new().fg(self.fg).bg(self.bg))
            .style(Style::new().bg(self.bg));
        let content_area = block.inner(dialogue_area).inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });
        block.render(dialogue_area, buf);

        match &self.kind {
            DialogueKind::ConfirmUnsavedChanges(state)
            | DialogueKind::Error(state) => state.render(content_area, buf),
            DialogueKind::FileSaveAs(state)
            | DialogueKind::ScriptInput(state)
            | DialogueKind::ScriptAutoInput(state) => {
                state.render(content_area, buf)
            }
        }
    }
}

trait DialogueState {
    fn on_event(&mut self, event: KeyEvent) -> DialogueAction;
}

const BUTTON_WIDTH: u16 = 10;

struct ButtonDialogueState {
    message: String,
    buttons: Vec<DialogueButton>,
    button_cursor: u8,
}

impl ButtonDialogueState {
    fn button_select_toggle(&mut self) {
        if !self.buttons.is_empty() {
            self.button_cursor ^= 1;
        }
    }
}

impl Widget for &ButtonDialogueState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::new()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area);
        sublayouts!([text_area, _, all_buttons_area] = layout);

        // Text
        Paragraph::new(Text::from(&*self.message))
            .wrap(Wrap { trim: false })
            .render(text_area, buf);

        // Button(s)
        let w = all_buttons_area.width;
        let btn_style = Style::default().bg(Color::Blue).fg(Color::White);
        let text_style = Style::default();
        let text_style_sel = text_style.underlined();
        if self.buttons.len() == 1 {
            let space_w = w.saturating_sub(BUTTON_WIDTH) / 2;
            let buttons_area = Layout::new()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Length(space_w),
                    Constraint::Length(BUTTON_WIDTH),
                    Constraint::Min(0),
                ])
                .split(all_buttons_area);
            let text = self.buttons[0].text().set_style(text_style_sel);
            Paragraph::new(text)
                .block(Block::default().style(btn_style))
                .alignment(Alignment::Center)
                .render(buttons_area[1], buf);
        } else {
            let space_w = w.saturating_sub(BUTTON_WIDTH * 2) / 3;
            let buttons_area = Layout::new()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Length(space_w),
                    Constraint::Length(BUTTON_WIDTH),
                    Constraint::Length(space_w),
                    Constraint::Length(BUTTON_WIDTH),
                    Constraint::Min(0),
                ])
                .split(all_buttons_area);
            let [text_style0, text_style1] = if self.button_cursor == 0 {
                [text_style_sel, text_style]
            } else {
                [text_style, text_style_sel]
            };
            Paragraph::new(self.buttons[0].text().set_style(text_style0))
                .block(Block::default().style(btn_style))
                .alignment(Alignment::Center)
                .render(buttons_area[1], buf);
            Paragraph::new(self.buttons[1].text().set_style(text_style1))
                .block(Block::default().style(btn_style))
                .alignment(Alignment::Center)
                .render(buttons_area[3], buf);
        }
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

            KeyCode::Tab
            | KeyCode::BackTab
            | KeyCode::Right
            | KeyCode::Left => {
                self.button_select_toggle();
                DialogueAction::None
            }

            _ => DialogueAction::None,
        }
    }
}

#[derive(Copy, Clone)]
enum DialogueButton {
    Ok,
    Yes,
    Cancel,
}

impl DialogueButton {
    fn text(&self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Yes => "YES",
            Self::Cancel => "CANCEL",
        }
    }
}

struct PromptDialogueState<'textarea> {
    button_state: ButtonDialogueState,
    input: TextArea<'textarea>,
}

impl Widget for &PromptDialogueState<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.button_state.render(area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
            .split(area.inner(&Margin {
                horizontal: 0,
                vertical: 2,
            }));
        sublayouts!([input_area, _] = layout);

        self.input.widget().render(input_area, buf);
    }
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
