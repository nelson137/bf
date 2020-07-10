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

    pub fn display(&self, highlight: bool, ascii_value: bool) -> String {
        let escaped = self.ascii().escape_default().to_string();
        let num = self.value().to_string();
        let value = if ascii_value {
            match self.ascii() {
                '\0' => r"\0",
                ' ' => "' '",
                '\t' | '\n' | '\r' | '!'..='~' => &escaped,
                _ => &num,
            }
        } else {
            &num
        };

        if highlight {
            // bg=Cyan fg=Black
            format!("\x1b[46m\x1b[30m{:^3}\x1b[0m", value)
        } else {
            format!("{:^3}", value)
        }
    }
}
