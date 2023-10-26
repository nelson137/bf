use std::vec::Vec;

use super::cell::Cell;

#[derive(Clone, Debug)]
pub struct Tape {
    cells: Vec<Cell>,
    cursor: usize,
}

impl Default for Tape {
    fn default() -> Self {
        Self {
            cells: vec![Cell::new(); 1],
            cursor: 0,
        }
    }
}

impl Tape {
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    fn get(&mut self, index: usize) -> &mut Cell {
        while index > self.cells.len() - 1 {
            self.cells.push(Cell::new());
        }
        &mut self.cells[index]
    }

    pub fn current(&mut self) -> &mut Cell {
        self.get(self.cursor)
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor += 1;
        // Force tape to be extended
        self.current();
    }
}
