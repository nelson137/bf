use ratatui::{symbols::line, widgets::BorderType};

pub const TAPE_BORDER_TYPE: BorderType = BorderType::Plain;

pub const TAPE_BORDER_SET: line::Set = line::NORMAL;

pub const TAPE_HORIZONTAL_BORDER_TOP: TapeBorderHorizontal =
    TapeBorderHorizontal {
        left_capped: TAPE_BORDER_SET.top_left,
        left_uncapped: TAPE_BORDER_SET.horizontal_down,
        middle: TAPE_BORDER_SET.horizontal,
        right_capped: TAPE_BORDER_SET.top_right,
        right_uncapped: TAPE_BORDER_SET.horizontal_down,
    };

pub const TAPE_HORIZONTAL_BORDER_BOTTOM: TapeBorderHorizontal =
    TapeBorderHorizontal {
        left_capped: TAPE_BORDER_SET.bottom_left,
        left_uncapped: TAPE_BORDER_SET.horizontal_up,
        middle: TAPE_BORDER_SET.horizontal,
        right_capped: TAPE_BORDER_SET.bottom_right,
        right_uncapped: TAPE_BORDER_SET.horizontal_up,
    };

pub struct TapeBorderHorizontal {
    left_capped: &'static str,
    left_uncapped: &'static str,
    middle: &'static str,
    right_capped: &'static str,
    right_uncapped: &'static str,
}

impl TapeBorderHorizontal {
    pub const fn left(&self, capped: bool) -> &'static str {
        if capped {
            self.left_capped
        } else {
            self.left_uncapped
        }
    }

    pub const fn middle(&self) -> &'static str {
        self.middle
    }

    pub const fn right(&self, capped: bool) -> &'static str {
        if capped {
            self.right_capped
        } else {
            self.right_uncapped
        }
    }
}

pub trait LineSetExts {
    fn top_divider(&self, width: usize, label: &str) -> String;
    fn middle_divider(&self, width: usize, label: &str) -> String;
    fn bottom_divider(&self, width: usize) -> String;
}

impl LineSetExts for line::Set {
    fn top_divider(&self, width: usize, label: &str) -> String {
        self.top_left.to_owned()
            + label
            + &self
                .horizontal
                .repeat(width.saturating_sub(2 + label.len()))
            + self.top_right
    }

    fn middle_divider(&self, width: usize, label: &str) -> String {
        self.vertical_right.to_owned()
            + label
            + &self
                .horizontal
                .repeat(width.saturating_sub(2 + label.len()))
            + self.vertical_left
    }

    fn bottom_divider(&self, width: usize) -> String {
        self.bottom_left.to_owned()
            + &self.horizontal.repeat(width.saturating_sub(2))
            + self.bottom_right
    }
}
