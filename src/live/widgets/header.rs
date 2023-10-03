use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Styled, Stylize},
    text::Span,
    widgets::{Paragraph, Widget},
};

use crate::{
    defaultable_builder,
    live::async_interpreter::Status,
    util::tui::{sublayouts, Frame, Spinner},
};

defaultable_builder! {
    #[derive(Default)]
    pub struct Header {
        is_dirty: bool,
        file_path: Option<String>,
        status: Status,
        spinner: Spinner,
    }
}

impl Header {
    pub fn render_(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(self, area);
    }
}

impl Widget for &Header {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(1),  // Dirty indicator
                Constraint::Length(1),  // Spacer (skip)
                Constraint::Min(0),     // Filename
                Constraint::Length(1),  // Spacer (skip)
                Constraint::Length(18), // Status (max status length)
                Constraint::Length(1),  // Spacer (skip)
                Constraint::Length(1),  // Spinner
            ])
            .split(area);
        sublayouts!(
            [indicator_area, _, fn_area, _, status_area, _, spinner_area] =
                layout
        );

        // Draw dirty indicator
        if self.is_dirty {
            Paragraph::new("*").render(indicator_area, buf);
        }

        // Draw filename
        Paragraph::new(match &self.file_path {
            Some(path) => Span::raw(path),
            None => "New File".add_modifier(Modifier::ITALIC),
        })
        .render(fn_area, buf);

        // Draw status
        let style = Style::default().add_modifier(Modifier::BOLD);
        let style = match self.status {
            Status::Done => Style::default(),
            Status::Running => style.fg(Color::Green),
            Status::WaitingForInput => style.fg(Color::Yellow),
            Status::Error(_) => style.fg(Color::Red),
            Status::FatalError(_) => style.fg(Color::Red),
        };
        Paragraph::new(self.status.to_string().set_style(style))
            .render(status_area, buf);

        // Draw spinner
        if self.status == Status::Running {
            self.spinner.render(spinner_area, buf);
        }
    }
}
