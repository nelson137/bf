use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{Paragraph, Widget},
};

const SPINNER: [&str; 4] = ["│", "╱", "─", "╲"];

#[derive(Clone, Copy, Default)]
pub struct Spinner(usize);

impl Spinner {
    pub fn tick(&mut self) {
        self.0 = (self.0 + 1) % SPINNER.len();
    }
}

impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(SPINNER[self.0]).render(area, buf);
    }
}
