use std::cmp::min;

use crate::util::EOL;

pub struct Field {
    lines: Vec<String>,
    cursor: (usize, usize),
    // selection: Option<(usize, usize)>,
}

impl Field {
    pub fn new() -> Self {
        Self::from("")
    }

    pub fn from(data: &str) -> Self {
        Self {
            lines: data.lines().map(|s| s.to_string()).collect(),
            cursor: (0, 0),
            // selection: None,
        }
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    pub fn cursor_line(&self) -> &String {
        &self.lines[self.cursor.0]
    }

    pub fn cursor_line_mut(&mut self) -> &mut String {
        &mut self.lines[self.cursor.0]
    }

    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter()
    }

    pub fn text(&self) -> String {
        self.lines
            .iter()
            .cloned()
            .map(|mut l| {
                l.push_str(EOL);
                l
            })
            .collect::<String>()
    }

    pub fn insert(&mut self, ch: char) {
        let cursor_x = self.cursor.1;
        self.cursor_line_mut().insert(cursor_x, ch);
        self.cursor.1 += 1;
    }

    pub fn enter(&mut self) {
        let x = self.cursor.1;
        let rest = self.cursor_line_mut().drain(x..).collect::<String>();
        self.cursor.0 += 1;
        self.cursor.1 = 0;
        self.lines.insert(self.cursor.0, rest);
    }

    fn cursor_x_clamped(&self) -> usize {
        min(self.cursor.1, self.cursor_line().len() - 1)
    }

    pub fn cursor_right(&mut self) {
        if self.cursor.1 < self.cursor_line().len() {
            self.cursor.1 += 1;
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor.1 = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor.1 = self.cursor_line().len();
    }

    pub fn cursor_up(&mut self) {
        if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.cursor_x_clamped();
        }
    }

    pub fn cursor_down(&mut self) {
        if self.cursor.0 < self.lines.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = self.cursor_x_clamped();
        }
    }

    pub fn cursor_top(&mut self) {
        self.cursor.0 = 0;
        self.cursor.1 = self.cursor_x_clamped();
    }

    pub fn cursor_bottom(&mut self) {
        self.cursor.0 = self.lines.len() - 1;
        self.cursor.1 = self.cursor_x_clamped();
    }

    /*
    fn remove_selection(&mut self) -> bool {
        if let Some((begin, len)) = self.selection {
            for _ in 0..len {
                self.data.remove(begin);
            }
            true
        } else {
            false
        }
    }
    */

    pub fn backspace(&mut self) {
        if self.cursor.1 == 0 {
            // Cursor is at col 0
            if self.cursor.0 != 0 {
                // Cursor is not at row 0
                let orig_line = self.cursor_line().clone();
                self.lines.remove(self.cursor.0);
                self.cursor.0 -= 1;
                self.cursor_end();
                self.cursor_line_mut().push_str(&orig_line);
            }
        } else {
            // Cursor is not at col 0
            self.cursor.1 -= 1;
            let cursor_x = self.cursor.1;
            self.cursor_line_mut().remove(cursor_x);
        }
    }

    pub fn delete(&mut self) {
        let (y, x) = self.cursor;
        let n_lines = self.lines.len();
        if x == self.cursor_line().len() {
            // Cursor is at last col
            if y != n_lines - 1 {
                // Cursor is not at last row
                let next_line = self.lines.get_mut(y + 1).unwrap().clone();
                self.cursor_line_mut().push_str(&next_line);
                self.lines.remove(y + 1);
            }
        } else {
            // Cursor is not at last col
            self.cursor_line_mut().remove(x);
        }
    }
}
