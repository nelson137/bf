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
        self.value() as char
    }

    pub fn set(&mut self, value: char) {
        self.0 = Wrapping(value as u8);
    }

    pub fn display(&self, ascii_value: bool) -> String {
        let escaped = self.ascii().escape_default().to_string();
        let num = self.value().to_string();
        let value = match self.ascii() {
            _ if ascii_value => &num,
            '\0' => r"\0",
            ' ' => "' '",
            '\t' | '\n' | '\r' | '!'..='~' => &escaped,
            _ => &num,
        };
        format!("{:^3}", value)
    }
}
