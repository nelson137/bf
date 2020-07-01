use std::process::exit;

pub fn die(msg: String) -> ! {
    eprintln!("bf: {}", msg);
    exit(1);
}

struct BoxCap {
    right: char,
    left: char,
    sep: char,
    spacer: char,
}

impl BoxCap {
    fn draw(&self, n_cells: usize) {
        print!("{0}{1}{1}{1}", self.left, self.spacer);
        for _ in 1..n_cells {
            print!("{0}{1}{1}{1}", self.sep, self.spacer);
        }
        println!("{}", self.right);
    }
}

pub struct BoxChars {
    top: BoxCap,
    bot: BoxCap,
    vert_sep: char,
}

impl BoxChars {
    pub fn draw(&self, contents: &[String]) {
        self.top.draw(contents.len());

        for c in contents {
            print!("{}{}", self.vert_sep, c);
        }
        println!("{}", self.vert_sep);

        self.bot.draw(contents.len());
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
