use std::iter;

use crossterm::event::{KeyCode, KeyEvent};

use crate::util::{
    common::{sha1_digest, Sha1Digest, StringExt, EOL},
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
    viewport: (usize, usize),
    cursor: (usize, usize),
}

impl TextArea {
    pub fn from(data: impl AsRef<str>, height: usize) -> Self {
        let lines = if data.as_ref().is_empty() {
            vec![String::new()]
        } else {
            data.as_ref().split('\n').map(|s| s.to_string()).collect()
        };
        Self {
            lines,
            viewport: (0, height),
            cursor: (0, 0),
        }
    }

    pub fn viewport_height(&self) -> usize {
        self.viewport.1
    }

    pub fn viewport_bounds(&self) -> (usize, usize) {
        let (begin, nlines) = self.viewport;
        let end = self.lines.len().min(begin + nlines);
        (begin, end)
    }

    pub fn viewport(&self) -> &[String] {
        let (begin, end) = self.viewport_bounds();
        &self.lines[begin..end]
    }

    pub fn viewport_mut(&mut self) -> &mut [String] {
        let (begin, end) = self.viewport_bounds();
        &mut self.lines[begin..end]
    }

    pub fn resize_viewport(&mut self, height: usize) {
        if self.viewport.1 == height || height == 0 {
            return;
        }
        self.viewport.1 = height;
        if self.viewport.0 > 0
            && self.lines.len() < self.viewport.0 + self.viewport.1
        {
            // Viewport goes past bottom & is not at top
            let new_vp0 = self.lines.len().saturating_sub(self.viewport.1);
            self.cursor.0 += self.viewport.0 - new_vp0;
            self.viewport.0 = new_vp0;
        } else if self.cursor.0 >= self.viewport.1 {
            // Cursor is past the end of the viewport
            self.viewport.0 += 1;
            self.cursor.0 = self.viewport.1.saturating_sub(1);
        }
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    pub fn cursor_line(&self) -> &String {
        &self.viewport()[self.cursor.0]
    }

    pub fn cursor_line_mut(&mut self) -> &mut String {
        let y = self.cursor.0;
        &mut self.viewport_mut()[y]
    }

    pub fn wrapped_numbered_lines(
        &self,
        width: usize,
    ) -> impl Iterator<Item = (Option<usize>, &str)> {
        (self.viewport.0 + 1..).zip(self.viewport()).flat_map(
            move |(n, line)| {
                iter::once(Some(n))
                    .chain(iter::repeat(None))
                    .zip(line.wrapped(width))
            },
        )
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
        if self.cursor.0 < self.viewport.1 - 1 {
            // Cursor is not at last row of viewport
            self.cursor.0 += 1;
        } else {
            // Cursor is at last row of viewport
            self.viewport.0 += 1;
        }
        self.cursor.1 = 0;
        self.lines.insert(self.viewport.0 + self.cursor.0, rest);
    }

    fn cursor_x_clamped(&self) -> usize {
        self.cursor_line().len().min(self.cursor.1)
    }

    pub fn cursor_right(&mut self) {
        if self.cursor.1 < self.cursor_line().len() {
            // Cursor is not at last col
            self.cursor.1 += 1;
        } else if self.cursor.0 < self.viewport.1 - 1 {
            // Cursor is at last col & not at last row of viewport
            if self.viewport.0 + self.cursor.0 < self.lines.len() - 1 {
                self.cursor.0 += 1;
                self.cursor.1 = 0;
            }
        } else if self.viewport.0 + self.viewport.1 < self.lines.len() {
            // Cursor is at last col & last row of viewport & viewport
            // is not at bottom
            self.viewport.0 += 1;
            self.cursor.1 = 0;
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor.1 > 0 {
            // Cursor is not at col 0
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            // Cursor is at col 0 & not at first row of viewport
            self.cursor.0 -= 1;
            self.cursor_end();
        } else if self.viewport.0 > 0 {
            // Cursor is at col 0 & at first row of viewport &
            // viewport is not at top
            self.viewport.0 -= 1;
            self.cursor_end();
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
            // Cursor is not at col 0
            self.cursor.0 -= 1;
            self.cursor.1 = self.cursor_x_clamped();
        } else if self.viewport.0 > 0 {
            // Cursor is at col 0 & viewport is not at top
            self.viewport.0 -= 1;
            self.cursor.1 = self.cursor_x_clamped();
        } else {
            // Cursor is at col 0 & viewport is at top
            self.cursor_home();
        }
    }

    pub fn cursor_down(&mut self) {
        if self.cursor.0 < (self.viewport.1 - 1).min(self.lines.len() - 1) {
            // Cursor is not at last line of viewport
            self.cursor.0 += 1;
            self.cursor.1 = self.cursor_x_clamped();
        } else if self.viewport.0 + self.viewport.1 < self.lines.len() {
            // Cursor is at last line of viewport & viewport is not at
            // bottom
            self.viewport.0 += 1;
            self.cursor.1 = self.cursor_x_clamped();
        } else {
            // Cursor is at last line of viewport & viewport is at
            // bottom
            self.cursor_end();
        }
    }

    pub fn cursor_top(&mut self) {
        self.viewport.0 = 0;
        self.cursor.0 = 0;
        self.cursor.1 = self.cursor_x_clamped();
    }

    pub fn cursor_bottom(&mut self) {
        self.viewport.0 = self.lines.len().saturating_sub(self.viewport.1);
        self.cursor.0 = self.viewport().len().saturating_sub(1);
        self.cursor.1 = self.cursor_x_clamped();
    }

    pub fn backspace(&mut self) {
        if self.cursor.1 > 0 {
            // Cursor is not at col 0
            self.cursor.1 -= 1;
            let x = self.cursor.1;
            self.cursor_line_mut().remove(x);
        } else if self.viewport.0 + self.cursor.0 > 0 {
            // Cursor is at col 0 & at not at top
            let original_line =
                self.lines.remove(self.viewport.0 + self.cursor.0);
            if self.viewport.0 + self.viewport.1 >= self.lines.len()
                && self.viewport.0 > 0
            {
                self.viewport.0 -= 1;
            } else {
                self.cursor.0 -= 1;
            }
            self.cursor_end();
            self.cursor_line_mut().push_str(&original_line)
        }
    }

    pub fn delete(&mut self) {
        let (y, x) = self.cursor;
        if x < self.cursor_line().len() {
            // Cursor is not at last col
            self.cursor_line_mut().remove(x);
        } else if self.viewport.0 + y < self.lines.len() - 1 {
            // Cursor is at last col & not at bottom
            let next_line = self.lines.remove(self.viewport.0 + y + 1);
            self.cursor_line_mut().push_str(&next_line);
            if self.viewport.0 + self.viewport.1 >= self.lines.len()
                && self.viewport.0 > 0
            {
                self.viewport.0 -= 1;
                self.cursor.0 += 1;
            }
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
