use std::num::Wrapping;

#[derive(Debug, Clone)]
pub struct Cell(Wrapping<u8>);

impl Cell {
    pub fn new() -> Self {
        Self(Wrapping(0))
    }

    pub fn inc(&mut self) {
        self.0 += Wrapping(1);
    }

    pub fn dec(&mut self) {
        self.0 -= Wrapping(1);
    }

    pub fn value(&self) -> u8 {
        (self.0).0
    }

    pub fn ascii(&self) -> char {
        (self.0).0 as char
    }

    pub fn set(&mut self, value: char) {
        self.0 = Wrapping(value as u8);
    }

    pub fn display(&self, highlight: bool) -> String {
        if highlight {
            // bg=Cyan fg=Black
            format!("\x1b[46m\x1b[30m{:^3}\x1b[0m", self.value())
        } else {
            format!("{:^3}", self.value())
        }
    }
}
