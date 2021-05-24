use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap}
};

use super::editable::{Editable, Field};

use crate::tui_util::{Frame, KeyEventExt};

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
    fn draw(&self, frame: &mut Frame, area: Rect);
    fn on_event(&mut self, event: KeyEvent) -> DialogueDecision;
}

#[derive(Clone, PartialEq)]
pub enum DialogueDecision {
    Waiting,
    No,
    Yes,
    Input(String)
}

#[derive(Clone)]
struct DialogueBox {
    bg: Color,
    fg: Color,
    title: String,
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
        let box_area = area.inner(&Margin { horizontal: 1, vertical: 0 });
        Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_type(tui::widgets::BorderType::Thick)
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

#[derive(Clone)]
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
            },
            msg: msg.into(),
            buttons: vec![DialogueButton::Cancel, DialogueButton::Yes],
            button_cursor: 0,
        }
    }

    fn button_select_toggle(&mut self) {
        if self.buttons.len() > 0 {
            self.button_cursor ^= 1;
        }
    }

}

impl Widget for ButtonDialogue {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.dialogue.render(area, buf);

        let content_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(area.inner(&Margin { horizontal: 3, vertical: 2 }));

        // Text
        Paragraph::new(self.msg)
            .wrap(Wrap { trim: false })
            .render(content_area[0], buf);

        // Button(s)
        let w = content_area[2].width;
        let btn_style = Style::default().bg(Color::Blue).fg(Color::White);
        let text_style = Style::default();
        let text_style_sel = text_style.clone()
            .add_modifier(Modifier::UNDERLINED);
        let buttons_area = Layout::default()
            .direction(Direction::Horizontal);
        if self.buttons.len() == 1 {
            let buttons_area = buttons_area.constraints(vec![
                    Constraint::Length((w - Self::BUTTON_WIDTH) / 2),
                    Constraint::Length(Self::BUTTON_WIDTH),
                    Constraint::Min(0),
                ])
                .split(content_area[2]);
            let text = Span::styled(self.buttons[0].text(), text_style_sel);
            Paragraph::new(text)
                .block(Block::default().style(btn_style))
                .alignment(Alignment::Center)
                .render(buttons_area[1], buf);
        } else {
            let space_w = w.saturating_sub(Self::BUTTON_WIDTH*2) / 3;
            let buttons_area = buttons_area.constraints(vec![
                    Constraint::Length(space_w),
                    Constraint::Length(Self::BUTTON_WIDTH),
                    Constraint::Length(space_w),
                    Constraint::Length(Self::BUTTON_WIDTH),
                    Constraint::Min(0),
                ])
                .split(content_area[2]);
            let [text_style0, text_style1] = if self.button_cursor == 0 {
                [text_style_sel, text_style]
            } else {
                [text_style, text_style_sel]
            };
            Paragraph::new(Span::styled(self.buttons[0].text(), text_style0))
                .block(Block::default().style(btn_style))
                .alignment(Alignment::Center)
                .render(buttons_area[1], buf);
            Paragraph::new(Span::styled(self.buttons[1].text(), text_style1))
                .block(Block::default().style(btn_style))
                .alignment(Alignment::Center)
                .render(buttons_area[3], buf);
        }
    }
}

impl Dialogue for ButtonDialogue {

    fn draw(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(self.clone(), area);
    }

    fn on_event(&mut self, event: KeyEvent) -> DialogueDecision {
        match event.code {
            KeyCode::Esc
                => return DialogueDecision::No,
            KeyCode::Char('c') if event.is_ctrl()
                => return DialogueDecision::No,
            KeyCode::Enter => {
                return if self.button_cursor == 0 {
                    DialogueDecision::No
                } else {
                    DialogueDecision::Yes
                };
            }

            KeyCode::Tab | KeyCode::Right |
            KeyCode::BackTab | KeyCode::Left => {
                self.button_select_toggle();
                DialogueDecision::Waiting
            }

            _ => DialogueDecision::Waiting,
        }
    }

}

#[derive(Clone)]
pub struct PromptStrDialogue {
    button_dialogue: ButtonDialogue,
    input: Field,
}

impl PromptStrDialogue {
    pub fn new<S>(title: S, prompt: S, default: Option<S>) -> Self
    where S: Into<String> {
        let input = if let Some(s) = default {
            Field::from(s)
        } else {
            Field::new()
        };
        Self {
            button_dialogue: ButtonDialogue {
                dialogue: DialogueBox {
                    bg: Color::Black,
                    fg: Color::Green,
                    title: title.into(),
                },
                msg: prompt.into(),
                buttons: vec![DialogueButton::Cancel, DialogueButton::Ok],
                button_cursor: 0,
            },
            input,
        }
    }
}

impl Dialogue for PromptStrDialogue {

    fn draw(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(self.clone(), area);
        frame.set_cursor(
            area.x + 4 + self.input.cursor() as u16,
            area.y + 5,
        );
    }

    fn on_event(&mut self, event: KeyEvent) -> DialogueDecision {
        match self.button_dialogue.on_event(event) {
            DialogueDecision::Waiting => {
                self.input.on_event(event);
                DialogueDecision::Waiting
            }
            DialogueDecision::Yes =>
                DialogueDecision::Input(self.input.text().into()),
            d => d,
        }
    }

}

impl Widget for PromptStrDialogue {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.button_dialogue.render(area, buf);

        let input_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area.inner(&Margin { horizontal: 3, vertical: 4 }))[0];

        Paragraph::new(self.input.text())
            .block(Block::default().borders(Borders::ALL))
            // .scroll((0u16, 1u16))
            .render(input_area, buf);
    }
}
