use std::{borrow::Cow, iter};

use crossterm::event::{KeyCode, KeyEvent};
use textwrap::{wrap, wrap_algorithms, Options};

use crate::util::{
    common::{sha1_digest, Sha1Digest, EOL},
    tui::KeyEventExt,
};

fn wrap_ragged(line: &str, width: usize) -> Vec<Cow<'_, str>> {
    wrap(
        line,
        Options::new(width).wrap_algorithm(wrap_algorithms::FirstFit),
    )
}

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

#[derive(Debug, Default, Copy, Clone)]
pub struct TextAreaCursor {
    pub y: usize,
    pub x: usize,
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TextAreaViewport {
    /// The index of the first line of the viewport.
    pub start: usize,

    /// The number of columns of the viewport.
    ///
    /// This directly affects the line wrapping.
    ///
    /// For example, if the width decreases then lines
    /// may wrap more and take up more rows in the viewport.
    /// Of course, the opposite may occur as well.
    pub width: usize,

    /// The number of rows of the viewport.
    ///
    /// Note that this is the number of rows, which doesn't
    /// necessarily correlate directly to lines in the file.
    ///
    /// For example, if each line in the viewport is long
    /// enough to wrap and take up exactly two rows, then
    /// the number of lines displayed will be half of the
    /// height.
    pub height: usize,
}

pub struct TextArea {
    lines: Vec<String>,
    viewport: TextAreaViewport,
    cursor: TextAreaCursor,
}

impl TextArea {
    pub fn from(data: impl AsRef<str>, width: usize, height: usize) -> Self {
        let lines = if data.as_ref().is_empty() {
            vec![String::new()]
        } else {
            data.as_ref().split('\n').map(|s| s.to_string()).collect()
        };
        Self {
            lines,
            viewport: TextAreaViewport {
                start: 0,
                width,
                height,
            },
            cursor: TextAreaCursor::default(),
        }
    }

    /// Return the number of (un-wrapped) lines.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// TODO: deleteme ??
    pub fn viewport(&self) -> TextAreaViewport {
        self.viewport
    }

    /// Return a tuple describing the range of the viewport.
    ///
    /// The tuple is `(viewport begin, viewport end)` where both
    /// values are indexes with the end value being one past the last
    /// in the range.
    fn viewport_bounds(&self) -> (usize, usize) {
        let start = self.viewport.start;
        let end = self.len().min(start + self.viewport.height);
        (start, end)
    }

    /// Return a slice of the (un-wrapped) lines that are visible in
    /// the viewport.
    pub fn viewport_lines(&self) -> &[String] {
        let (begin, end) = self.viewport_bounds();
        &self.lines[begin..end]
    }

    /// Return a mutable slice of the (un-wrapped) lines that are
    /// visible in the viewport.
    pub fn viewport_lines_mut(&mut self) -> &mut [String] {
        let (begin, end) = self.viewport_bounds();
        &mut self.lines[begin..end]
    }

    // TODO: this instead of Self::viewport() ??
    pub fn viewport_cursor(&self) -> TextAreaCursor {
        // TODO: figure out how to cache the wrapped rows
        //         - hash the text ??
        //         - keep a dirty bool ??
        let rows = self.wrapped_numbered_rows();
        let y = {
            let mut y = 0;
            let mut line_count = 0;
            for row in rows {
                if (row.1).0.is_some() {
                    line_count += 1;
                }
                if line_count > self.cursor.y {
                    break;
                }
                y += 1;
            }
            y + (self.cursor.x / self.viewport.width)
        };
        TextAreaCursor {
            y,
            x: self.cursor.x % self.viewport.width,
        }
    }

    /// Update the width and height of the viewport, adjusting its
    /// starting position and height as necessary.
    ///
    /// If the height grows and the viewport is not at he bottom,
    /// then more lines will be added.
    ///
    /// If the height grows and the viewport is at the bottom, then
    /// the starting position will decrease and the height will
    /// increase to include more lines from above the viewport, until
    /// the viewport grows large enough to fit the entire buffer.
    ///
    /// If the width changes, then the line wrapping will be
    /// recalculated so that the number of line rows is known to
    /// correctly update the starting position if necessary.
    pub fn resize_viewport(&mut self, width: usize, height: usize) {
        if self.viewport.height == height && self.viewport.width == width {
            return;
        }
        if height == 0 || width == 0 {
            return;
        }

        self.viewport.width = width;
        self.viewport.height = height;

        let viewport_lines = self.viewport_lines();

        let line_row_counts = viewport_lines
            .iter()
            .map(|l| wrap_ragged(l, width).len())
            .collect::<Vec<_>>();

        let mut row_count = 0;
        for rc in line_row_counts {
            if row_count >= height {
                break;
            }
            row_count += rc;
        }

        if self.viewport.start > 0 && row_count < height {
            // Viewport is not at top & goes past bottom
            // (all of the lines from viewport_lines(), wrapped or
            // not, don't fill up the `height`)
            let new_vp_start = self.len().saturating_sub(height);
            self.cursor.y += self.viewport.start - new_vp_start;
            self.viewport.start = new_vp_start;
        } else if self.cursor.y >= height {
            // Cursor is past the end of the viewport
            self.viewport.start += 1;
            self.cursor.y = height.saturating_sub(1);
        }
    }

    pub fn cursor_line(&self) -> &String {
        &self.viewport_lines()[self.cursor.y]
    }

    pub fn cursor_line_mut(&mut self) -> &mut String {
        let y = self.cursor.y;
        &mut self.viewport_lines_mut()[y]
    }

    pub fn wrapped_numbered_rows(
        &self,
    ) -> impl Iterator<Item = (usize, (Option<usize>, Cow<'_, str>))> {
        (self.viewport.start + 1..)
            .zip(self.viewport_lines())
            .flat_map(|(n, line)| {
                iter::once(Some(n))
                    .chain(iter::repeat(None))
                    .zip(wrap_ragged(line, self.viewport.width))
            })
            .enumerate()
            .take_while(|(i, (maybe_n, _))| {
                *i < self.viewport.height || maybe_n.is_none()
            })
    }

    pub fn text(&self) -> String {
        self.lines.join(EOL)
    }

    pub fn hash(&self) -> Sha1Digest {
        sha1_digest(self.text())
    }

    fn seek_eol_down(&mut self) {
        let row_counts = self
            .viewport_lines()
            .iter()
            .take(self.cursor.y)
            .map(|line| wrap_ragged(line, self.viewport.width).len())
            .collect::<Vec<_>>();

        let mut count: usize = row_counts.iter().sum();

        let current_line_count =
            wrap_ragged(self.cursor_line(), self.viewport.width).len();
        let h = self.viewport.height.saturating_sub(current_line_count);

        let mut i = 0;
        while i < row_counts.len() && count > h {
            count = count.saturating_sub(row_counts[i]);
            i += 1;
        }

        self.viewport.start += i;
        self.cursor.y = self.cursor.y.saturating_sub(i);
    }

    pub fn insert(&mut self, ch: char) {
        if ch.is_ascii_graphic() || ch.is_ascii_whitespace() {
            let cursor_x = self.cursor.x;
            self.cursor_line_mut().insert(cursor_x, ch);
            self.cursor.x += 1;
        }
    }

    pub fn enter(&mut self) {
        let x = self.cursor.x;
        let rest = self.cursor_line_mut().drain(x..).collect::<String>();
        if self.cursor.y < self.viewport.height - 1 {
            // Cursor is not at last row of viewport
            self.cursor.y += 1;
        } else {
            // Cursor is at last row of viewport
            self.viewport.start += 1;
        }
        self.cursor.x = 0;
        self.lines.insert(self.viewport.start + self.cursor.y, rest);
        self.seek_eol_down();
    }

    fn cursor_x_clamped(&self) -> usize {
        self.cursor_line().len().min(self.cursor.x)
    }

    pub fn cursor_right(&mut self) {
        if self.cursor.x < self.cursor_line().len() {
            // Cursor is not at last col
            self.cursor.x += 1;
        } else if self.cursor.y < self.viewport.height - 1 {
            // Cursor is at last col & not at last row of viewport
            if self.viewport.start + self.cursor.y < self.len() - 1 {
                self.cursor.y += 1;
                self.cursor.x = 0;
            }
            self.seek_eol_down();
        } else if self.viewport.start + self.viewport.height < self.len() {
            // Cursor is at last col & last row of viewport & viewport
            // is not at bottom
            self.viewport.start += 1;
            self.cursor.x = 0;
            self.seek_eol_down();
        }
    }

    pub fn cursor_left(&mut self) {
        if self.cursor.x > 0 {
            // Cursor is not at col 0
            self.cursor.x -= 1;
        } else if self.cursor.y > 0 {
            // Cursor is at col 0 & not at first row of viewport
            self.cursor.y -= 1;
            self.cursor_end();
        } else if self.viewport.start > 0 {
            // Cursor is at col 0 & at first row of viewport &
            // viewport is not at top
            self.viewport.start -= 1;
            self.cursor_end();
        }
    }

    pub fn cursor_home(&mut self) {
        self.cursor.x = 0;
    }

    pub fn cursor_end(&mut self) {
        self.cursor.x = self.cursor_line().len();
    }

    pub fn cursor_up(&mut self) {
        if self.cursor.y > 0 {
            // Cursor is not at col 0
            self.cursor.y -= 1;
            self.cursor.x = self.cursor_x_clamped();
        } else if self.viewport.start > 0 {
            // Cursor is at col 0 & viewport is not at top
            self.viewport.start -= 1;
            self.cursor.x = self.cursor_x_clamped();
        } else {
            // Cursor is at col 0 & viewport is at top
            self.cursor_home();
        }
    }

    pub fn cursor_down(&mut self) {
        if self.cursor.y < (self.viewport.height - 1).min(self.len() - 1) {
            // Cursor is not at last line of viewport
            self.cursor.y += 1;
            self.cursor.x = self.cursor_x_clamped();
            self.seek_eol_down();
        } else if self.viewport.start + self.viewport.height < self.len() {
            // Cursor is at last line of viewport & viewport is not at
            // bottom
            self.viewport.start += 1;
            self.cursor.x = self.cursor_x_clamped();
            self.seek_eol_down();
        } else {
            // Cursor is at last line of viewport & viewport is at
            // bottom
            self.cursor_end();
        }
    }

    pub fn cursor_top(&mut self) {
        self.viewport.start = 0;
        self.cursor.y = 0;
        self.cursor.x = self.cursor_x_clamped();
    }

    pub fn cursor_bottom(&mut self) {
        self.viewport.start = self.len().saturating_sub(self.viewport.height);
        self.cursor.y = self.viewport_lines().len().saturating_sub(1);
        self.cursor.x = self.cursor_x_clamped();
    }

    pub fn backspace(&mut self) {
        let TextAreaViewport {
            width,
            height,
            start,
        } = self.viewport;

        if self.cursor.x > 0 {
            // Cursor is not at col 0
            self.cursor.x -= 1;
            let x = self.cursor.x;
            self.cursor_line_mut().remove(x);
        } else if start == 0 {
            // Cursor is at col 0 & viewport at top
            if self.cursor.y > 0 {
                let original_line = self.lines.remove(start + self.cursor.y);
                self.cursor.y -= 1;
                self.cursor_end();
                self.cursor_line_mut().push_str(&original_line);
            }
        } else if self.cursor.y == 0 {
            // Cursor is at col 0 & viewport not at top & cursor at
            // row 0
            let original_line = self.lines.remove(start);
            self.viewport.start -= 1;
            self.cursor_end();
            self.cursor_line_mut().push_str(&original_line);
        } else if self.viewport.start + self.cursor.y > 0 {
            // Cursor is at col 0 & not on first line

            let original_line = self.lines.remove(start + self.cursor.y);

            let mut line_count = 0;
            let mut row_count = 0;
            for line in self.viewport_lines() {
                let rc = wrap_ragged(line, width).len();
                if row_count >= height {
                    break;
                }
                line_count += 1;
                row_count += rc;
            }

            if start + line_count < self.len() {
                // Viewport is not at bottom
                if self.cursor.y == 0 {
                    self.viewport.start -= 1;
                } else {
                    self.cursor.y -= 1;
                }
                self.cursor_end();
                self.cursor_line_mut().push_str(&original_line);
            } else {
                // Viewport is at bottom
                self.cursor.y -= 1;
                self.cursor_end();
                let orig_line_rc = {
                    let chunks = wrap_ragged(&original_line, width);
                    if chunks.len() == 1 && chunks[0].is_empty() {
                        0
                    } else {
                        chunks.len()
                    }
                };
                let old_line_rc = {
                    let chunks = wrap_ragged(self.cursor_line(), width);
                    if chunks.len() == 1 && chunks[0].is_empty() {
                        0
                    } else {
                        chunks.len()
                    }
                };
                self.cursor_line_mut().push_str(&original_line);
                let new_line_rc = wrap_ragged(self.cursor_line(), width).len();
                let above_rc =
                    wrap_ragged(&self.lines[start - 1], width).len();
                if row_count - orig_line_rc - old_line_rc
                    + new_line_rc
                    + above_rc
                    <= height
                {
                    // The line above the viewport can be added without
                    // pushing the new current line too far down
                    self.viewport.start -= 1;
                    self.cursor.y += 1;
                }
            }
        }
    }

    pub fn delete(&mut self) {
        let TextAreaCursor { y, x } = self.cursor;
        if x < self.cursor_line().len() {
            // Cursor is not at last col
            self.cursor_line_mut().remove(x);
        } else if self.viewport.start + y < self.len() - 1 {
            // Cursor is at last col & not at bottom
            let next_line = self.lines.remove(self.viewport.start + y + 1);
            self.cursor_line_mut().push_str(&next_line);
            if self.viewport.start + self.viewport.height >= self.len()
                && self.viewport.start > 0
            {
                self.viewport.start -= 1;
                self.cursor.y += 1;
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
