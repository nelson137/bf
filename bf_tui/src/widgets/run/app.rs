use bf::interpreter::Interpreter;
use ratatui::{
    prelude::{Buffer, Constraint, Layout, Rect},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{sublayouts, widgets::ChunkedTapeWidget};

#[derive(Default)]
pub struct AppWidgetState {
    pub height: u16,
}

pub struct AppWidget<'interpreter> {
    pub show_tape: bool,
    pub ascii_values: bool,
    pub interpreter: &'interpreter Interpreter,
}

impl StatefulWidget for AppWidget<'_> {
    type State = AppWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let output = self.interpreter.output();
        let output_lines = output.split_terminator('\n').count() as u16;

        let tape_widget = ChunkedTapeWidget::new(
            &self.interpreter.tape,
            area.width as i32,
            self.ascii_values,
        );
        let tape_height = 3 * tape_widget.len() as u16;

        state.height = tape_height + output_lines;

        let layout = Layout::vertical([
            Constraint::Length(tape_height),
            Constraint::Length(output_lines),
        ])
        .split(area);
        sublayouts!([tape_area, output_area] = layout);

        tape_widget.render(tape_area, buf);

        if !output.is_empty() {
            Paragraph::new(output).render(output_area, buf);
        }
    }
}
