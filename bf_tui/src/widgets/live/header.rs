use bf_utils::defaultable_builder;
use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Styled, Stylize},
    text::Span,
    widgets::{Paragraph, Widget},
};

use crate::{async_interpreter::Status, sublayouts, widgets::Spinner};

defaultable_builder! {
    #[derive(Default)]
    pub struct Header<'path> {
        is_dirty: bool,
        file_path: Option<&'path str>,
        status: Status,
        spinner: Spinner,
    }
}

impl Widget for &Header<'_> {
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
        Paragraph::new(match self.file_path {
            Some(path) => Span::raw(path),
            None => "New File".italic(),
        })
        .render(fn_area, buf);

        // Draw status
        let style = Style::default().bold();
        let style = match self.status {
            Status::Done => Style::default(),
            Status::Running => style.fg(Color::Green),
            Status::WaitingForInput => style.fg(Color::Yellow),
            Status::Error(_) | Status::FatalError(_) => style.fg(Color::Red),
        };
        Paragraph::new(self.status.to_string().set_style(style))
            .render(status_area, buf);

        // Draw spinner
        if self.status == Status::Running {
            self.spinner.render(spinner_area, buf);
        }
    }
}
