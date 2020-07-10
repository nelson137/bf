use std::process::exit;

pub fn die(msg: String) -> ! {
    eprintln!("bf: {}", msg);
    exit(1);
}

pub fn ends_with_eol(s: &str) -> bool {
    s.ends_with('\n') || s.ends_with("\r\n")
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

pub const BOX_UNICODE: BoxChars = BoxChars {
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
