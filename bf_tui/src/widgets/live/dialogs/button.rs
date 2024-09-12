use ratatui::{
    layout::Flex,
    prelude::{Alignment, Buffer, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Paragraph, Widget},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DialogButton {
    Ok,
    Yes,
    Cancel,
}

pub const BUTTON_WIDTH: u16 = 10;

impl DialogButton {
    pub const fn text(self) -> &'static str {
        /*
         * NOTE: Keep `BUTTON_WIDTH` up to date with the length of longest text.
         */
        match self {
            Self::Ok => "OK",
            Self::Yes => "YES",
            Self::Cancel => "CANCEL",
        }
    }

    pub const fn is_affirmative(self) -> bool {
        match self {
            Self::Ok | Self::Yes => true,
            Self::Cancel => false,
        }
    }
}

pub struct DialogButtonWidget {
    kind: DialogButton,
    fg: Color,
    selected: bool,
}

impl DialogButtonWidget {
    pub const fn new(kind: DialogButton, fg: Color, selected: bool) -> Self {
        Self { kind, fg, selected }
    }
}

impl Widget for DialogButtonWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::new().style(if self.selected {
            Style::new()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Blue)
                .bg(self.fg)
        } else {
            Style::new().fg(self.fg)
        });

        Paragraph::new(self.kind.text())
            .block(block)
            .alignment(Alignment::Center)
            .render(area, buf);

        buf[(area.left(), area.y)].set_char('[');
        buf[(area.right() - 1, area.y)].set_char(']');
    }
}

pub struct ButtonRowWidget<'buttons> {
    buttons: &'buttons [DialogButton],
    cursor: Option<u8>,
    fg: Color,
}

impl<'buttons> ButtonRowWidget<'buttons> {
    pub const fn new(
        buttons: &'buttons [DialogButton],
        cursor: Option<u8>,
        fg: Color,
    ) -> Self {
        Self {
            buttons,
            cursor,
            fg,
        }
    }
}

impl Widget for ButtonRowWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let constraint = Constraint::Length(BUTTON_WIDTH);
        let layout = Layout::horizontal(vec![constraint; self.buttons.len()])
            .flex(Flex::End)
            .spacing(2)
            .horizontal_margin(2)
            .split(area);

        let buttons = self.buttons.iter().copied();
        let button_areas = layout.iter().copied();
        for ((i, button), button_area) in buttons.enumerate().zip(button_areas)
        {
            let selected = matches!(self.cursor, Some(c) if c as usize == i);
            DialogButtonWidget::new(button, self.fg, selected)
                .render(button_area, buf);
        }
    }
}
