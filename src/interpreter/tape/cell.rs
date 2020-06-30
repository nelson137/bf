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
}
