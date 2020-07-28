use pancurses::{init_pair, COLOR_PAIR};

#[derive(Clone, Copy)]
pub enum Style {
    Cursor = 1,
    ControlHint,
    StatusOk,
    StatusErr,
}

impl Style {
    pub fn init(&self, fg: i16, bg: i16) {
        init_pair(*self as i16, fg, bg);
    }

    pub fn get(&self) -> u64 {
        COLOR_PAIR(*self as u64)
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
