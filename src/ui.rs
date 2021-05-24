use pancurses::{chtype, has_colors, Window};

pub trait WindowStyleDo {
    fn style_do<F: FnOnce() -> i32>(&self, attr: chtype, func: F);
}

impl WindowStyleDo for Window {
    fn style_do<F: FnOnce() -> i32>(&self, attr: chtype, func: F) {
        if has_colors() {
            self.attron(attr);
        }
        func();
        if has_colors() {
            self.attroff(attr);
        }
    }
}

pub struct BoxLid {
    pub right: char,
    pub left: char,
    pub sep: char,
    pub spacer: char,
}

pub struct BoxChars {
    pub top: BoxLid,
    pub bot: BoxLid,
    pub vert_sep: char,
}

pub const TAPE_UNICODE: BoxChars = BoxChars {
    top: BoxLid {
        left: '┌',
        right: '┐',
        sep: '┬',
        spacer: '─',
    },
    bot: BoxLid {
        left: '└',
        right: '┘',
        sep: '┴',
        spacer: '─',
    },
    vert_sep: '│',
};
