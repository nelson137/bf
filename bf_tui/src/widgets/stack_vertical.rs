use ratatui::{
    prelude::{Buffer, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::lines::{LineSetExts, TAPE_BORDER_SET, TAPE_BORDER_TYPE};

pub struct VerticalStack<'titles, const SECTIONS: usize> {
    titles: [&'titles str; SECTIONS],
    area: Rect,
    divider_areas: Vec<Rect>,
    section_areas: Vec<Rect>,
    section_content_areas: Vec<Rect>,
}

impl<'titles, const SECTIONS: usize> VerticalStack<'titles, SECTIONS> {
    pub fn new(
        heights: [Constraint; SECTIONS],
        titles: [&'titles str; SECTIONS],
        area: Rect,
    ) -> Self {
        assert!(SECTIONS > 0, "VerticalStack must have at least 1 section");
        let n_dividers = SECTIONS + 1;
        let n_areas = SECTIONS + n_dividers;

        let mut constraints = Vec::with_capacity(n_areas);
        constraints.push(Constraint::Length(1));
        for h in heights {
            constraints.push(h);
            constraints.push(Constraint::Length(1));
        }

        let all_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        let mut divider_areas = Vec::with_capacity(n_dividers);
        let mut section_areas = Vec::with_capacity(SECTIONS);
        let mut section_content_areas = Vec::with_capacity(SECTIONS);

        let content_block =
            Block::new().borders(Borders::LEFT | Borders::RIGHT);

        divider_areas.push(all_areas[0]);
        for i in (1..n_areas).step_by(2) {
            section_areas.push(all_areas[i]);
            section_content_areas.push(content_block.inner(all_areas[i]));
            divider_areas.push(all_areas[i + 1]);
        }

        Self {
            titles,
            area,
            divider_areas,
            section_areas,
            section_content_areas,
        }
    }

    pub fn areas(&self) -> [Rect; SECTIONS] {
        self.section_content_areas
            .clone()
            .try_into()
            .expect("The number of section content areas does not match the number of sections in this VerticalStack")
    }
}

impl<const SECTIONS: usize> Widget for VerticalStack<'_, SECTIONS> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        assert!(area == self.area, "VerticalStack area for rendering is different than the one given to VerticalStack::new");

        let width = area.width as usize;

        for i in 0..self.divider_areas.len() {
            let area = self.divider_areas[i];
            let divider = match i {
                0 => TAPE_BORDER_SET.top_divider(width, self.titles[i]),
                x if x < self.divider_areas.len() - 1 => {
                    TAPE_BORDER_SET.middle_divider(width, self.titles[i])
                }
                _ => TAPE_BORDER_SET.bottom_divider(width),
            };
            Paragraph::new(divider).render(area, buf);
        }

        for area in self.section_areas {
            Block::new()
                .border_type(TAPE_BORDER_TYPE)
                .borders(Borders::LEFT | Borders::RIGHT)
                .render(area, buf);
        }
    }
}
