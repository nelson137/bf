use crossterm::event::{KeyCode, KeyEvent};
use ratatui_textarea::{CursorMove, TextArea};
use sha1::{Digest, Sha1};

use crate::util::{common::Sha1Digest, tui::KeyEventExt};

pub trait TextAreaExts {
    fn bytes(&self) -> impl Iterator<Item = u8>;

    fn hash(&self) -> Sha1Digest;

    fn len(&self) -> usize;

    fn on_event_multi_line(&mut self, event: KeyEvent);
    fn on_event_single_line(&mut self, event: KeyEvent);

    fn to_string(&self) -> String;
}

impl TextAreaExts for TextArea<'_> {
    fn bytes(&self) -> impl Iterator<Item = u8> {
        self.lines().iter().flat_map(|l| l.bytes())
    }

    fn hash(&self) -> Sha1Digest {
        let mut digest = Sha1::new();

        for line in self.lines() {
            digest.update(line);
        }

        digest.finalize().into()
    }

    fn len(&self) -> usize {
        self.lines().len()
    }

    fn on_event_multi_line(&mut self, event: KeyEvent) {
        match event.code {
            // Cursor movement
            KeyCode::Left => self.move_cursor(CursorMove::Back),
            KeyCode::Right => self.move_cursor(CursorMove::Forward),
            KeyCode::Up => self.move_cursor(CursorMove::Up),
            KeyCode::Down => self.move_cursor(CursorMove::Down),
            KeyCode::Home => self.move_cursor(CursorMove::Head),
            KeyCode::End => self.move_cursor(CursorMove::End),
            KeyCode::PageUp => self.move_cursor(CursorMove::Top),
            KeyCode::PageDown => self.move_cursor(CursorMove::Bottom),

            // Insertions
            KeyCode::Enter => self.insert_newline(),
            KeyCode::Tab => {
                self.insert_tab();
            }
            KeyCode::Char(c) if !event.is_ctrl() && !event.is_alt() => {
                self.insert_char(c)
            }

            // Deletions
            KeyCode::Backspace => {
                self.delete_char();
            }
            KeyCode::Delete => {
                self.delete_next_char();
            }

            // Others
            _ => (),
        }
    }

    fn on_event_single_line(&mut self, event: KeyEvent) {
        match event.code {
            // Cursor movement
            KeyCode::Left => self.move_cursor(CursorMove::Back),
            KeyCode::Right => self.move_cursor(CursorMove::Forward),
            KeyCode::Home => self.move_cursor(CursorMove::Head),
            KeyCode::End => self.move_cursor(CursorMove::End),

            // Insertions
            KeyCode::Char(c) if !event.is_ctrl() && !event.is_alt() => {
                self.insert_char(c)
            }

            // Deletions
            KeyCode::Backspace => {
                self.delete_char();
            }
            KeyCode::Delete => {
                self.delete_next_char();
            }

            // Others
            _ => (),
        }
    }

    fn to_string(&self) -> String {
        self.lines().join("")
    }
}
