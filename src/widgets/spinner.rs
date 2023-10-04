use ratatui::{
    prelude::{Buffer, Rect},
    widgets::{Paragraph, Widget},
};

const SPINNER: &str = "│╱─╲";

#[derive(Clone, Copy, Default)]
pub struct Spinner(usize);

impl Spinner {
    fn state_boundaries() -> impl Iterator<Item = usize> {
        (0..=SPINNER.len()).filter(|i| SPINNER.is_char_boundary(*i))
    }

    pub fn tick(&mut self) {
        let n_states = Self::state_boundaries().count() - 1;
        self.0 = (self.0 + 1) % n_states;
    }

    fn get(&self) -> &'static str {
        let mut indexes = Vec::with_capacity(2);
        indexes.extend(Self::state_boundaries().skip(self.0).take(2));
        &SPINNER[indexes[0]..indexes[1]]
    }
}

impl Widget for Spinner {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.get()).render(area, buf);
    }
}
