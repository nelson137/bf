use std::{
    cell::{Ref, RefCell},
    io,
    rc::Rc,
};

use delegate::delegate;
use ratatui::{
    backend::{Backend, TestBackend},
    buffer,
    prelude::Rect,
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

#[derive(Clone)]
pub struct MyTestBackend {
    backend: Rc<RefCell<TestBackend>>,
}

impl MyTestBackend {
    pub fn new(width: u16, height: u16) -> Self {
        let backend = TestBackend::new(width, height);
        Self {
            backend: Rc::new(RefCell::new(backend)),
        }
    }

    pub fn get(&self) -> Ref<TestBackend> {
        self.backend.borrow()
    }
}

impl Backend for MyTestBackend {
    delegate! {
        to self.backend.borrow_mut() {
            fn draw<'a, I>(&mut self, content: I) -> Result<(), io::Error>
            where
                I: Iterator<Item = (u16, u16, &'a buffer::Cell)>;

            fn hide_cursor(&mut self) -> Result<(), io::Error>;

            fn show_cursor(&mut self) -> Result<(), io::Error>;

            fn get_cursor(&mut self) -> Result<(u16, u16), io::Error>;

            fn set_cursor(&mut self, x: u16, y: u16) -> Result<(), io::Error>;

            fn clear(&mut self) -> Result<(), io::Error>;

            fn size(&self) -> Result<Rect, io::Error>;

            fn flush(&mut self) -> Result<(), io::Error>;
        }
    }
}

pub fn terminal(
    width: u16,
    height: u16,
) -> (Terminal<MyTestBackend>, MyTestBackend) {
    let backend = MyTestBackend::new(width, height);
    let term =
        Terminal::new(backend.clone()).expect("failed to create terminal");
    (term, backend)
}
