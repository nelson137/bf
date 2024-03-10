use ratatui::{
    symbols::line,
    text::{Line, Span},
    widgets::BorderType,
};

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
    fn top_divider<'label>(
        &self,
        width: usize,
        label: &Line<'label>,
    ) -> Line<'label>;
    fn middle_divider<'label>(
        &self,
        width: usize,
        label: &Line<'label>,
    ) -> Line<'label>;
    fn bottom_divider(&self, width: usize) -> Line;
}

impl LineSetExts for line::Set {
    fn top_divider<'label>(
        &self,
        width: usize,
        label: &Line<'label>,
    ) -> Line<'label> {
        let mut spans = Vec::with_capacity(1 + label.iter().count() + 2);
        spans.push(Span::raw(self.top_left));
        spans.extend(label.iter().cloned());
        spans.push(
            self.horizontal
                .repeat(width.saturating_sub(2 + label.width()))
                .into(),
        );
        spans.push(self.top_right.into());
        spans.into()
    }

    fn middle_divider<'label>(
        &self,
        width: usize,
        label: &Line<'label>,
    ) -> Line<'label> {
        let mut spans = Vec::with_capacity(1 + label.iter().count() + 2);
        spans.push(Span::raw(self.vertical_right));
        spans.extend(label.iter().cloned());
        spans.push(
            self.horizontal
                .repeat(width.saturating_sub(2 + label.width()))
                .into(),
        );
        spans.push(self.vertical_left.into());
        spans.into()
    }

    fn bottom_divider(&self, width: usize) -> Line {
        vec![
            Span::raw(self.bottom_left),
            self.horizontal.repeat(width.saturating_sub(2)).into(),
            self.bottom_right.into(),
        ]
        .into()
    }
}
