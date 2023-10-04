use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style, Styled},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};
use ratatui_textarea::TextArea;

use crate::util::tui::{sublayouts, KeyEventExt};

use super::textarea::TextAreaExts;

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

pub trait Dialogue: Widget {
    fn get_reason(&self) -> Reason;
    fn set_reason(&mut self, reason: Reason);
    fn set_action(&mut self, f: Box<dyn FnOnce()>);
    fn run_action(&mut self);
    fn draw(&self, area: Rect, buf: &mut Buffer);
    fn on_event(&mut self, event: KeyEvent) -> Decision;
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    None,
    Info,
    Confirm,
    Filename,
    Input,
    AutoInput,
}

#[derive(Clone, PartialEq)]
pub enum Decision {
    Waiting,
    No,
    Yes,
    Input(String),
}

struct DialogueBox {
    bg: Color,
    fg: Color,
    title: String,
    reason: Reason,
    action: Option<Box<dyn FnOnce()>>,
}

impl DialogueBox {
    fn get_reason(&self) -> Reason {
        self.reason
    }

    fn set_reason(&mut self, reason: Reason) {
        self.reason = reason;
    }

    fn set_action(&mut self, f: Box<dyn FnOnce()>) {
        self.action = Some(f);
    }

    fn run_action(&mut self) {
        if let Some(action) = self.action.take() {
            action();
        }
    }

    fn clone_data(&self) -> Self {
        Self {
            action: None,
            title: self.title.clone(),
            ..*self
        }
    }
}

impl Widget for DialogueBox {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        // Margin box (single column on left and right)
        Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_style(Style::default().fg(self.bg).bg(self.bg))
            .render(area, buf);

        // Box with outline and title
        let box_area = area.inner(&Margin {
            horizontal: 1,
            vertical: 0,
        });
        Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Thick)
            .border_style(Style::default().fg(self.fg).bg(self.bg))
            .style(Style::default().bg(self.bg))
            .render(box_area, buf);
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

pub struct ButtonDialogue {
    dialogue: DialogueBox,
    msg: String,
    buttons: Vec<DialogueButton>,
    button_cursor: u8,
}

impl ButtonDialogue {
    const BUTTON_WIDTH: u16 = 10;

    pub fn error<S: Into<String>>(msg: S) -> Self {
        Self {
            dialogue: DialogueBox {
                bg: Color::Black,
                fg: Color::Red,
                title: " Error ".to_string(),
                reason: Reason::None,
                action: None,
            },
            msg: msg.into(),
            buttons: vec![DialogueButton::Ok],
            button_cursor: 0,
        }
    }

    pub fn confirm<S: Into<String>>(msg: S) -> Self {
        Self {
            dialogue: DialogueBox {
                bg: Color::Black,
                fg: Color::Yellow,
                title: " Confirm ".to_string(),
                reason: Reason::None,
                action: None,
            },
            msg: msg.into(),
            buttons: vec![DialogueButton::Cancel, DialogueButton::Yes],
            button_cursor: 0,
        }
    }

    fn clone_data(&self) -> Self {
        Self {
            dialogue: self.dialogue.clone_data(),
            msg: self.msg.clone(),
            buttons: self.buttons.clone(),
            ..*self
        }
    }

    fn button_select_toggle(&mut self) {
        if !self.buttons.is_empty() {
            self.button_cursor ^= 1;
        }
    }
}

impl Widget for ButtonDialogue {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.dialogue.render(area, buf);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area.inner(&Margin {
                horizontal: 3,
                vertical: 2,
            }));
        sublayouts!([text_area, _, all_buttons_area] = layout);

        // Text
        Paragraph::new(self.msg)
            .wrap(Wrap { trim: false })
            .render(text_area, buf);

        // Button(s)
        let w = all_buttons_area.width;
        let btn_style = Style::default().bg(Color::Blue).fg(Color::White);
        let text_style = Style::default();
        let text_style_sel = text_style.add_modifier(Modifier::UNDERLINED);
        if self.buttons.len() == 1 {
            let space_w = w.saturating_sub(Self::BUTTON_WIDTH) / 2;
            let buttons_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Length(space_w),
                    Constraint::Length(Self::BUTTON_WIDTH),
                    Constraint::Min(0),
                ])
                .split(all_buttons_area);
            let text = Span::styled(self.buttons[0].text(), text_style_sel);
            Paragraph::new(text)
                .block(Block::default().style(btn_style))
                .alignment(Alignment::Center)
                .render(buttons_area[1], buf);
        } else {
            let space_w = w.saturating_sub(Self::BUTTON_WIDTH * 2) / 3;
            let buttons_area = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                    Constraint::Length(space_w),
                    Constraint::Length(Self::BUTTON_WIDTH),
                    Constraint::Length(space_w),
                    Constraint::Length(Self::BUTTON_WIDTH),
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

impl Dialogue for ButtonDialogue {
    fn get_reason(&self) -> Reason {
        self.dialogue.get_reason()
    }

    fn set_reason(&mut self, reason: Reason) {
        self.dialogue.set_reason(reason);
    }

    fn set_action(&mut self, f: Box<dyn FnOnce()>) {
        self.dialogue.set_action(f);
    }

    fn run_action(&mut self) {
        self.dialogue.run_action();
    }

    fn draw(&self, area: Rect, buf: &mut Buffer) {
        self.clone_data().render(area, buf);
    }

    fn on_event(&mut self, event: KeyEvent) -> Decision {
        match event.code {
            KeyCode::Esc => Decision::No,
            KeyCode::Char('c') if event.is_ctrl() => Decision::No,
            KeyCode::Enter => {
                if self.button_cursor == 0 {
                    Decision::No
                } else {
                    Decision::Yes
                }
            }

            KeyCode::Tab
            | KeyCode::Right
            | KeyCode::BackTab
            | KeyCode::Left => {
                self.button_select_toggle();
                Decision::Waiting
            }

            _ => Decision::Waiting,
        }
    }
}

pub struct PromptStrDialogue<'textarea> {
    button_dialogue: ButtonDialogue,
    input: TextArea<'textarea>,
}

impl PromptStrDialogue<'_> {
    pub fn new<S>(title: S, prompt: S, default: Option<S>) -> Self
    where
        S: Into<String>,
    {
        let mut input = if let Some(s) = default {
            TextArea::new(vec![s.into()])
        } else {
            TextArea::new(vec![])
        };
        input.set_block(Block::default().borders(Borders::ALL));
        Self {
            button_dialogue: ButtonDialogue {
                dialogue: DialogueBox {
                    bg: Color::Black,
                    fg: Color::Green,
                    title: title.into(),
                    reason: Reason::None,
                    action: None,
                },
                msg: prompt.into(),
                buttons: vec![DialogueButton::Cancel, DialogueButton::Ok],
                button_cursor: 0,
            },
            input,
        }
    }

    fn clone_data(&self) -> Self {
        Self {
            button_dialogue: self.button_dialogue.clone_data(),
            input: self.input.clone(),
        }
    }
}

impl Dialogue for PromptStrDialogue<'_> {
    fn get_reason(&self) -> Reason {
        self.button_dialogue.get_reason()
    }

    fn set_reason(&mut self, reason: Reason) {
        self.button_dialogue.set_reason(reason);
    }

    fn set_action(&mut self, f: Box<dyn FnOnce()>) {
        self.button_dialogue.set_action(f);
    }

    fn run_action(&mut self) {
        self.button_dialogue.run_action();
    }

    fn draw(&self, area: Rect, buf: &mut Buffer) {
        self.clone_data().render(area, buf);
    }

    fn on_event(&mut self, event: KeyEvent) -> Decision {
        match self.button_dialogue.on_event(event) {
            Decision::Waiting => {
                self.input.on_event_single_line(event);
                Decision::Waiting
            }
            Decision::Yes => Decision::Input(self.input.to_string()),
            d => d,
        }
    }
}

impl Widget for PromptStrDialogue<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.button_dialogue.render(area, buf);

        let input_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(3), Constraint::Min(0)])
            .split(area.inner(&Margin {
                horizontal: 3,
                vertical: 4,
            }))[0];

        self.input.widget().render(input_area, buf);
    }
}
