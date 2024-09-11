use std::{
    io::stdout,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use bf_tui::events::{BfEvent, EventQueue};
use crossterm::{
    event::{Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    prelude::{Backend, CrosstermBackend},
    text::{Line, Text},
    widgets::Paragraph,
    Frame, Terminal, TerminalOptions, Viewport,
};

use crate::utils::read::read_script;

pub use self::cli::InlineScrollCli;

mod cli;

macro_rules! event_matches {
    ($evt:expr, $code:expr) => {
        matches!(
            $evt,
            crossterm::event::KeyEvent {
                kind: crossterm::event::KeyEventKind::Press,
                modifiers: crossterm::event::KeyModifiers::NONE,
                state: crossterm::event::KeyEventState::NONE,
                code
            } if code == $code
        )
    };
}

fn run(args: InlineScrollCli) -> Result<()> {
    let script: String = read_script(args.infile.as_ref())?
        .iter()
        .flat_map(|l| l.chars().filter(|c| !c.is_whitespace()))
        .collect();

    let mut output: Vec<String> = Vec::default();
    let mut id = 0;

    enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    let options = TerminalOptions {
        viewport: Viewport::Inline(backend.size()?.height),
    };
    let mut terminal = Terminal::with_options(backend, options)?;
    terminal.clear()?;

    let event_queue = EventQueue::with_ticks(args.delay);
    let mut quit = false;

    let ret = try {
        while !quit {
            for event in event_queue.pop_all() {
                match event {
                    BfEvent::Tick => {
                        let now =
                            SystemTime::now().duration_since(UNIX_EPOCH)?;
                        output.push(format!(
                            "{ts:.03} {script} {id}",
                            ts = now.as_millis() as f64 / 1000.0,
                        ));
                        id += 1;

                        let lines = output.iter().map(String::as_str);
                        let len = output.len() as u16;
                        let size = terminal.size()?;
                        let scroll = len.saturating_sub(size.height);
                        terminal.draw(|frame| draw(frame, lines, scroll))?;
                    }
                    BfEvent::Input(Event::Key(key_evt)) => {
                        if event_matches!(key_evt, KeyCode::Esc) {
                            quit = true;
                        }
                    }
                    BfEvent::Input(_) => {}
                }
            }
        }
    };

    disable_raw_mode()?;
    println!();

    ret
}

fn draw<'s>(
    frame: &mut Frame,
    lines: impl Iterator<Item = &'s str>,
    scroll: u16,
) {
    let area = frame.area();
    let lines: Vec<Line> = lines.map(Line::from).collect();
    let text = Text::from(lines);
    let widget = Paragraph::new(text).scroll((scroll, 0));
    frame.render_widget(widget, area);
}
