use button::DialogueButton;
use crossterm::event::KeyEvent;
use ratatui::{
    prelude::{Buffer, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Clear, Padding, Widget},
};
use tui_textarea::TextArea;

use self::drop_shadow::DropShadowWidget;

mod button;
mod drop_shadow;

bf_utils::barrel_module! {
    pub mod error;
    pub mod file_save_as;
    pub mod script_auto_input;
    pub mod script_input;
    pub mod unsaved_changes;
}

pub struct Dialogue<'dialog> {
    title: &'static str,
    bg: Color,
    primary: Color,
    fg: Color,
    dialogue: Box<dyn AppDialogue + 'dialog>,
}

impl Dialogue<'_> {
    const DEFAULT_BG: Color = Color::Reset;
    const DEFAULT_FG: Color = Color::White;

    pub fn on_event(&mut self, event: KeyEvent) -> DialogueCommand {
        self.dialogue.on_event(event)
    }
}

impl Widget for &Dialogue<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        let block = Block::bordered()
            .title(self.title)
            .title_style(Style::new().bg(self.bg).fg(self.primary))
            .border_type(BorderType::Thick)
            .border_style(Style::new().bg(self.bg).fg(self.primary))
            .style(Style::new().bg(self.bg).fg(self.fg))
            .padding(Padding::uniform(1));
        let content_area = block.inner(area);
        block.render(area, buf);

        self.dialogue.render(content_area, buf);

        DropShadowWidget::new(2, 2).render(area, buf);
    }
}

pub trait AppDialogue {
    fn on_event(&mut self, event: KeyEvent) -> DialogueCommand;
    fn render(&self, area: Rect, buf: &mut Buffer);
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

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let centered_vert_area = Layout::vertical(vec![
        Constraint::Fill(1),
        Constraint::Percentage(percent_y),
        Constraint::Fill(1),
    ])
    .split(area)[1];

    Layout::horizontal(vec![
        Constraint::Fill(1),
        Constraint::Percentage(percent_x),
        Constraint::Fill(1),
    ])
    .split(centered_vert_area)[1]
}

struct DialogueFocusController {
    order: Vec<DialogueFocus>,
    index: usize,
}

impl DialogueFocusController {
    const fn new(order: Vec<DialogueFocus>) -> Self {
        Self { order, index: 0 }
    }

    fn to_buttons(&self) -> Vec<DialogueButton> {
        let mut buttons: Vec<_> = self
            .order
            .iter()
            .filter_map(|f| match *f {
                DialogueFocus::Input => None,
                DialogueFocus::Button { index, kind } => Some((index, kind)),
            })
            .collect();
        buttons.sort_by_key(|b| b.0);
        buttons.into_iter().map(|b| b.1).collect()
    }

    //
    // Getters
    //

    fn get(&self) -> DialogueFocus {
        self.order[self.index]
    }

    fn is_input(&self) -> bool {
        self.get() == DialogueFocus::Input
    }

    fn should_submit(&self) -> bool {
        match self.get() {
            DialogueFocus::Input => true,
            DialogueFocus::Button { kind, .. } => kind.is_affirmative(),
        }
    }

    fn button_cursor(&self) -> Option<u8> {
        match self.get() {
            DialogueFocus::Button { index, .. } => Some(index),
            _ => None,
        }
    }

    //
    // Mutators
    //

    fn next(&mut self) {
        self.index = (self.index + 1) % self.order.len();
    }

    fn prev(&mut self) {
        self.index = (self.index + self.order.len() - 1) % self.order.len();
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DialogueFocus {
    #[default]
    Input,
    Button {
        index: u8,
        kind: DialogueButton,
    },
}

impl DialogueFocus {
    const fn button(index: u8, kind: DialogueButton) -> Self {
        Self::Button { index, kind }
    }
}

fn render_input(
    input: &mut TextArea,
    area: Rect,
    buf: &mut Buffer,
    focused: bool,
) {
    let (cursor_style, border_style) = if focused {
        (Style::new().reversed(), Style::new().fg(Color::White))
    } else {
        (Style::new(), Style::new().fg(Color::DarkGray))
    };
    let block = Block::bordered().border_style(border_style);
    input.set_block(block);
    input.set_cursor_style(cursor_style);
    input.render(area, buf);
}
