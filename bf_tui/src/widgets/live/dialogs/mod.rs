use button::DialogButton;
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

pub struct Dialog<'dialog> {
    title: &'static str,
    bg: Color,
    primary: Color,
    fg: Color,
    dialog: Box<dyn AppDialog + 'dialog>,
}

impl Dialog<'_> {
    const DEFAULT_BG: Color = Color::Reset;
    const DEFAULT_FG: Color = Color::White;

    pub fn on_event(&mut self, event: KeyEvent) -> DialogCommand {
        self.dialog.on_event(event)
    }
}

impl Widget for &Dialog<'_> {
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

        self.dialog.render(content_area, buf);

        DropShadowWidget::new(2, 2).render(area, buf);
    }
}

pub trait AppDialog {
    fn on_event(&mut self, event: KeyEvent) -> DialogCommand;
    fn render(&self, area: Rect, buf: &mut Buffer);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DialogCommand {
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

struct DialogFocusController {
    order: Vec<DialogFocus>,
    index: usize,
}

impl DialogFocusController {
    const fn new(order: Vec<DialogFocus>) -> Self {
        Self { order, index: 0 }
    }

    fn to_buttons(&self) -> Vec<DialogButton> {
        let mut buttons: Vec<_> = self
            .order
            .iter()
            .filter_map(|f| match *f {
                DialogFocus::Input => None,
                DialogFocus::Button { index, kind } => Some((index, kind)),
            })
            .collect();
        buttons.sort_by_key(|b| b.0);
        buttons.into_iter().map(|b| b.1).collect()
    }

    //
    // Getters
    //

    fn get(&self) -> DialogFocus {
        self.order[self.index]
    }

    fn is_input(&self) -> bool {
        self.get() == DialogFocus::Input
    }

    fn should_submit(&self) -> bool {
        match self.get() {
            DialogFocus::Input => true,
            DialogFocus::Button { kind, .. } => kind.is_affirmative(),
        }
    }

    fn button_cursor(&self) -> Option<u8> {
        match self.get() {
            DialogFocus::Button { index, .. } => Some(index),
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
enum DialogFocus {
    #[default]
    Input,
    Button {
        index: u8,
        kind: DialogButton,
    },
}

impl DialogFocus {
    const fn button(index: u8, kind: DialogButton) -> Self {
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
