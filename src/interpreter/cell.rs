use std::num::Wrapping;

use crate::util::BoxLid;

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
}

pub struct CellDisplay<'a> {
    cell: &'a Cell,
    is_highlighted: bool,
}

impl<'a> CellDisplay<'a> {
    pub fn new(cell: &'a Cell) -> Self {
        Self {
            cell,
            is_highlighted: false,
        }
    }

    pub fn is_highlighted(&self) -> bool {
        self.is_highlighted
    }

    pub fn highlight(&mut self) {
        self.is_highlighted = true;
    }

    pub fn display_lid(
        &self,
        lid: &BoxLid,
        left_cap: bool,
        right: Option<bool>,
    ) -> String {
        let mut buf = String::new();

        buf.push(if left_cap { lid.left } else { lid.sep });
        (0..3).for_each(|_| buf.push(lid.spacer));
        if let Some(is_cap) = right {
            buf.push(if is_cap { lid.right } else { lid.sep });
        }

        buf
    }

    pub fn display(&self, ascii_value: bool) -> String {
        let escaped = self.cell.ascii().escape_default().to_string();
        let num = self.cell.value().to_string();
        let value = match self.cell.ascii() {
            _ if !ascii_value => &num,
            '\0' => r"\0",
            ' ' => "' '",
            '\t' | '\n' | '\r' | '!'..='~' => &escaped,
            _ => &num,
        };
        format!("{:^3}", value)
    }
}
