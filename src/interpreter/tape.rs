use itertools::Itertools;

use crate::util::DrawStyle;

mod cell;
use cell::Cell;

#[derive(Debug)]
pub struct Tape {
    cells: Vec<Cell>,
    cursor: usize,
}

impl Tape {
    pub fn new() -> Self {
        Self {
            cells: vec![Cell::new(); 1],
            cursor: 0,
        }
    }

    fn get(&mut self, index: usize) -> &mut Cell {
        while index > self.cells.len() - 1 {
            self.cells.push(Cell::new());
        }
        unsafe { self.cells.get_unchecked_mut(index) }
    }

    pub fn current(&mut self) -> &mut Cell {
        self.get(self.cursor)
    }

    pub fn left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn right(&mut self) {
        self.cursor += 1;
        // Force tape to be extended
        self.current();
    }

    pub fn draw(&mut self, width: u32, style: &DrawStyle) -> String {
        // Each cell is 4 wide + the extra vertical separator
        let cells_per_chunk = ((width - 1) / 4) as usize;

        let n_chunks =
            (self.cells.len() as f64 / cells_per_chunk as f64).ceil() as usize;

        self.cells
            .iter()
            .enumerate()
            .map(|(i, c)| c.display(i == self.cursor))
            .chunks(cells_per_chunk)
            .into_iter()
            .enumerate()
            .map(|(i, chunk)| {
                style.draw_box(
                    &chunk.collect::<Vec<_>>(),
                    i == 0,
                    i == n_chunks - 1,
                )
            })
            .collect::<String>()
    }
}
