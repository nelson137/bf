use std::process::exit;

pub fn die(msg: String) -> ! {
    eprintln!("bf: {}", msg);
    exit(1);
}

pub fn ends_with_eol(s: &str) -> bool {
    s.ends_with('\n') || s.ends_with("\r\n")
}

macro_rules! repeat_action {
    ($n:literal, $action:stmt) => {
        for __ in 0..$n {
            $action
        }
    };
}

struct BoxCap {
    right: char,
    left: char,
    sep: char,
    spacer: char,
}

impl BoxCap {
    fn draw(&self, n_cells: usize, left_cap: bool, right_cap: bool) -> String {
        let mut buf = String::new();

        buf.push(if left_cap { self.left } else { self.sep });
        repeat_action!(3, buf.push(self.spacer));

        for _ in 1..n_cells {
            buf.push(self.sep);
            repeat_action!(3, buf.push(self.spacer));
        }

        buf.push(if right_cap { self.right } else { self.sep });
        buf.push('\n');

        buf
    }
}

pub struct DrawStyle {
    top: BoxCap,
    bot: BoxCap,
    vert_sep: char,
}

impl DrawStyle {
    pub fn draw_box(
        &self,
        contents: &[String],
        left_cap: bool,
        right_cap: bool,
    ) -> String {
        let mut buf = String::new();

        buf.push_str(&self.top.draw(contents.len(), left_cap, right_cap));

        for c in contents {
            buf.push(self.vert_sep);
            buf.push_str(c);
        }
        buf.push(self.vert_sep);
        buf.push('\n');

        buf.push_str(&self.bot.draw(contents.len(), left_cap, right_cap));

        buf
    }
}

pub const DRAW_STYLE_ASCII: DrawStyle = DrawStyle {
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

pub const DRAW_STYLE_UNICODE: DrawStyle = DrawStyle {
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
