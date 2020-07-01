use std::process::exit;

pub fn die(msg: String) -> ! {
    eprintln!("bf: {}", msg);
    exit(1);
}

pub fn ends_with_eol(s: &str) -> bool {
    s.ends_with('\n') || s.ends_with("\r\n")
}

struct BoxCap {
    right: char,
    left: char,
    sep: char,
    spacer: char,
}

impl BoxCap {
    fn draw(&self, n_cells: usize) -> String {
        let mut buf = String::new();
        buf.push(self.left);
        buf.push(self.spacer);
        buf.push(self.spacer);
        buf.push(self.spacer);
        for _ in 1..n_cells {
            buf.push(self.sep);
            buf.push(self.spacer);
            buf.push(self.spacer);
            buf.push(self.spacer);
        }
        buf.push(self.right);
        buf.push('\n');
        buf
    }
}

pub struct BoxChars {
    top: BoxCap,
    bot: BoxCap,
    vert_sep: char,
}

impl BoxChars {
    pub fn draw(&self, contents: &[String]) -> String {
        let mut buf = String::new();

        buf.push_str(&self.top.draw(contents.len()));

        for c in contents {
            buf.push(self.vert_sep);
            buf.push_str(c);
        }
        buf.push(self.vert_sep);
        buf.push('\n');

        buf.push_str(&self.bot.draw(contents.len()));

        buf
    }
}

pub const BOX_CHARS_ASCII: BoxChars = BoxChars {
    top: BoxCap {
        left: '+',
        right: '+',
        sep: '+',
        spacer: '-',
    },
    bot: BoxCap {
        left: '+',
        right: '+',
        sep: '+',
        spacer: '-',
    },
    vert_sep: '|',
};

pub const BOX_CHARS_UNICODE: BoxChars = BoxChars {
    top: BoxCap {
        left: '┌',
        right: '┐',
        sep: '┬',
        spacer: '─',
    },
    bot: BoxCap {
        left: '└',
        right: '┘',
        sep: '┴',
        spacer: '─',
    },
    vert_sep: '│',
};
