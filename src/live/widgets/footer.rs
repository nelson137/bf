use ratatui::{
    style::{Color, Style, Styled},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

const CONTROLS: [[&str; 2]; 6] = [
    ["^S", "Save"],
    ["^X", "Save As"],
    ["^C", "Quit"],
    ["^A", "Toggle ASCII"],
    ["F1", "Set Input"],
    ["F2", "Set Auto-Input"],
];

pub struct Footer;

impl Widget for Footer {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let keys_style = Style::default().bg(Color::Cyan).fg(Color::Black);

        let text = Line::from(
            CONTROLS
                .iter()
                .flat_map(|[keys, desc]| {
                    vec![
                        keys.set_style(keys_style),
                        Span::from(":"),
                        Span::from(*desc),
                        Span::from("  "),
                    ]
                })
                .collect::<Vec<_>>(),
        );

        Paragraph::new(text).render(area, buf);
    }
}
