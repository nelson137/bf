use ratatui::{
    style::{Color, Stylize},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

const KEY_BINDINGS: &[(&str, &str)] = &[
    ("^S", "Save"),
    ("^X", "Save As"),
    ("^C", "Quit"),
    ("^A", "Toggle ASCII"),
    ("F1", "Set Input"),
    ("F2", "Set Auto-Input"),
];

pub struct Footer;

impl Widget for Footer {
    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
    ) {
        let sections = KEY_BINDINGS
            .iter()
            .flat_map(|(keys, desc)| {
                [
                    keys.bg(Color::Cyan).fg(Color::Black),
                    Span::from(":"),
                    Span::from(*desc),
                    Span::from("  "),
                ]
            })
            .collect::<Vec<_>>();

        Paragraph::new(Line::from(sections)).render(area, buf);
    }
}
