use std::process::exit;

use pancurses::{init_pair, COLOR_PAIR};

#[cfg(windows)]
pub const EOL: &str = "\r\n";
#[cfg(not(windows))]
pub const EOL: &str = "\n";

pub fn die(msg: String) -> ! {
    eprintln!("bf: {}", msg);
    exit(1);
}

pub fn get_width(width: Option<usize>) -> i32 {
    (match width {
        Some(w) => w,
        None => match term_size::dimensions() {
            Some((w, _h)) if w > 5 => w,
            _ => 65, // Wide enough for 16 cells
        },
    }) as i32
}

pub fn is_valid_width(value: String) -> Result<(), String> {
    match value.parse::<i64>() {
        Ok(n) => {
            if n < 5 {
                Err("value must be an integer > 5".to_string())
            } else {
                Ok(())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

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
