use bf::interpreter::Tape;
use ratatui::{
    prelude::{Buffer, Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::{
    async_interpreter::{
        State as InterpreterState, Status as InterpreterStatus,
    },
    sublayouts,
    widgets::{Spinner, VerticalStack},
};

use super::{
    dialogs::{centered_rect, Dialog},
    Footer, Header, TapeViewport, TapeViewportState,
};

pub struct AppWidget<'app, 'textarea, Editor: Widget> {
    pub term_width: usize,
    pub term_height: usize,
    pub dialog: Option<&'app Dialog<'textarea>>,
    pub is_dirty: bool,
    pub file_path: Option<&'app str>,
    pub spinner: Spinner,
    pub async_interpreter: InterpreterState,
    pub tape_viewport: TapeViewportState,
    pub editor: Editor,
}

impl<Editor: Widget> Widget for AppWidget<'_, '_, Editor> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Min(7),
            Constraint::Length(1),
        ])
        .split(area);
        sublayouts!([header_area, content_area, footer_area] = layout);

        draw_header(
            header_area,
            buf,
            self.is_dirty,
            self.file_path,
            self.async_interpreter.status,
            self.spinner,
        );

        draw_content(
            content_area,
            buf,
            &self.async_interpreter.tape,
            &mut self.tape_viewport,
            self.editor,
            &self.async_interpreter.output,
        );

        draw_footer(footer_area, buf);

        if let Some(dialog) = &self.dialog {
            let dialog_area = centered_rect(50, 50, area);
            dialog.render(dialog_area, buf);
        }
    }
}

fn draw_header(
    area: Rect,
    buf: &mut Buffer,
    is_dirty: bool,
    file_path: Option<&str>,
    interpreter_status: InterpreterStatus,
    spinner: Spinner,
) {
    Header::default()
        .is_dirty(is_dirty)
        .file_path(file_path)
        .status(interpreter_status)
        .spinner(spinner)
        .render(area, buf);
}

fn draw_content(
    area: Rect,
    buf: &mut Buffer,
    tape: &Tape,
    tape_state: &mut TapeViewportState,
    editor: impl Widget,
    output: &[u8],
) {
    let output = String::from_utf8_lossy(output);
    let output_lines = output.split_terminator('\n').count() as u16;

    let tape_title = Line::raw(" Tape ");
    let output_title = if output.ends_with('\n') {
        Line::raw(" Output ")
    } else {
        vec![Span::raw(" Output "), "(no EOL) ".italic().dark_gray()].into()
    };
    let code_title = Line::raw(" Code ");

    let stack = VerticalStack::<3>::new(
        [
            Constraint::Length(3),            // Tape
            Constraint::Min(1),               // Editor
            Constraint::Length(output_lines), // Output
        ],
        [&tape_title, &code_title, &output_title],
        area,
    );

    let [tape_area, editor_area, output_area] = stack.areas();

    stack.render(area, buf);

    // Tape
    TapeViewport::new(tape).render(tape_area, buf, tape_state);

    // Editor
    editor.render(editor_area, buf);

    // Output
    if !output.is_empty() {
        Paragraph::new(output).render(output_area, buf);
    }
}

fn draw_footer(area: Rect, buf: &mut Buffer) {
    Footer.render(area, buf);
}
