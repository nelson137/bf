pub struct Field {
    data: String,
    cursor: usize,
    selection: Option<(usize, usize)>,
}

impl Field {
    pub fn new() -> Self {
        Self {
            data: String::new(),
            cursor: 0,
            selection: None,
        }
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn data(&self) -> &str {
        &self.data
    }

    pub fn insert(&mut self, ch: char) {
        self.data.insert(self.cursor, ch);
        self.cursor += 1;
    }

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

    pub fn backspace(&mut self) {
        if !self.remove_selection() && self.cursor > 0 {
            self.data.remove(self.cursor - 1);
            self.cursor -= 1;
        }
    }

    pub fn delete(&mut self) {
        if !self.remove_selection() && self.cursor < self.data.len() {
            self.data.remove(self.cursor);
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
}
