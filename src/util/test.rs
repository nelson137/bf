use ratatui::{
    backend::TestBackend,
    style::{Modifier, Style},
    Terminal,
};

use crate::interpreter::{Interpreter, Tape};

pub const CELL_STYLE_NORMAL: Style = Style::new();
pub const CELL_STYLE_CURSOR: Style =
    Style::new().add_modifier(Modifier::REVERSED);

pub fn tape_from_script(script: &str) -> Tape {
    let mut int = Interpreter::new(script.bytes(), [].into(), None);
    for _ in &mut int {}
    int.tape
}

pub fn terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).expect("failed to create terminal")
}
