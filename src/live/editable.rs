use std::cmp::min;

use crossterm::event::{KeyCode, KeyEvent};

use crate::util::{
    common::{sha1_digest, Sha1Digest, EOL},
    tui::KeyEventExt,
};

pub trait Editable {
    fn on_event(&mut self, event: KeyEvent);
}

#[derive(Clone)]
pub struct Field {
    data: String,
    cursor: usize,
}

impl Field {
    pub fn new() -> Self {
        Self::from("")
    }

    pub fn from<S: Into<String>>(data: S) -> Self {
        let data = data.into();
        let cursor = data.len();
        Self { data, cursor }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn text(&self) -> &str {
        &self.data
    }

    pub fn insert(&mut self, ch: char) {
        if ch.is_ascii_graphic() || ch.is_ascii_whitespace() {
            self.data.insert(self.cursor, ch);
            self.cursor += 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor < self.data.len() {
            self.cursor += 1;
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor = self.data.len();
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.data.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.data.len() {
            self.data.remove(self.cursor);
        }
    }
}

impl Editable for Field {
    fn on_event(&mut self, event: KeyEvent) {
        match event.code {
            // Cursor movement
            KeyCode::Left => self.cursor_left(),
            KeyCode::Right => self.cursor_right(),
            KeyCode::Home => self.cursor_home(),
            KeyCode::End => self.cursor_end(),

            // Insertions
            KeyCode::Char(c) if !event.is_ctrl() && !event.is_alt() => {
                self.insert(c)
            }

            // Deletions
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),

            // Others
            _ => (),
        }
    }
}

pub struct TextArea {
    lines: Vec<String>,
    cursor: (usize, usize),
}

impl TextArea {
    pub fn from<S: AsRef<str>>(data: S) -> Self {
        let lines = if data.as_ref().is_empty() {
            vec![String::new()]
        } else {
            data.as_ref().split('\n').map(|s| s.to_string()).collect()
        };
        Self {
            lines,
            cursor: (0, 0),
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
        self.lines.join(EOL)
    }

    pub fn hash(&self) -> Sha1Digest {
        sha1_digest(self.text())
    }

    pub fn insert(&mut self, ch: char) {
        if ch.is_ascii_graphic() || ch.is_ascii_whitespace() {
            let cursor_x = self.cursor.1;
            self.cursor_line_mut().insert(cursor_x, ch);
            self.cursor.1 += 1;
        }
    }

    pub fn enter(&mut self) {
        let x = self.cursor.1;
        let rest = self.cursor_line_mut().drain(x..).collect::<String>();
        self.cursor.0 += 1;
        self.cursor.1 = 0;
        self.lines.insert(self.cursor.0, rest);
    }

    fn cursor_x_clamped(&self) -> usize {
        let cursor_line_len = self.cursor_line().len();
        if cursor_line_len == 0 {
            0
        } else {
            min(self.cursor.1, cursor_line_len)
        }
    }

    pub fn cursor_right(&mut self) {
        let line_len = self.cursor_line().len();
        if self.cursor.1 == line_len && self.cursor.0 < self.lines.len() - 1 {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        } else if self.cursor.1 < line_len {
            self.cursor.1 += 1;
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor.1 == 0 && self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor_end();
        } else if self.cursor.1 > 0 {
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

impl Editable for TextArea {
    fn on_event(&mut self, event: KeyEvent) {
        match event.code {
            // Cursor movement
            KeyCode::Left => self.cursor_left(),
            KeyCode::Right => self.cursor_right(),
            KeyCode::Up => self.cursor_up(),
            KeyCode::Down => self.cursor_down(),
            KeyCode::Home => self.cursor_home(),
            KeyCode::End => self.cursor_end(),
            KeyCode::PageUp => self.cursor_top(),
            KeyCode::PageDown => self.cursor_bottom(),

            // Insertions
            KeyCode::Enter => self.enter(),
            KeyCode::Tab => self.insert('\t'),
            KeyCode::Char(c) if !event.is_ctrl() && !event.is_alt() => {
                self.insert(c)
            }

            // Deletions
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),

            // Others
            _ => (),
        }
    }
}
