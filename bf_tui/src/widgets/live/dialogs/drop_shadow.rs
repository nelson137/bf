use ratatui::{
    prelude::{Buffer, Rect},
    style::{Color, Style},
    widgets::Widget,
};

pub struct DropShadowWidget {
    offset_x: u16,
    offset_y: u16,
}

impl DropShadowWidget {
    pub const fn new(offset_x: u16, offset_y: u16) -> Self {
        Self { offset_x, offset_y }
    }
}

impl Widget for DropShadowWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let bottom_area = Rect {
            y: area.y + area.height,
            x: area.x + self.offset_x,
            width: area.width,
            height: self.offset_y,
        };
        render_section(bottom_area, buf);

        let right_area = Rect {
            y: area.y + self.offset_y,
            x: area.x + area.width,
            width: self.offset_x,
            height: area.height.saturating_sub(self.offset_y),
        };
        render_section(right_area, buf);

        fn render_section(area: Rect, buf: &mut Buffer) {
            for y in area.top()..area.bottom() {
                for x in area.left()..area.right() {
                    let cell = &mut buf[(x, y)];
                    cell.reset();
                    cell.set_style(Style::new().bg(Color::Black));
                }
            }
        }
    }
}
