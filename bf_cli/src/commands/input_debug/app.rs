use std::{io::stdout, thread, time::Duration};

use anyhow::Result;
use bf_tui::{
    events::{BfEvent, EventQueue},
    sublayouts, Terminal,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph, Row, Table},
    Frame,
};

use super::{cli::InputDebugCli, state::State};

pub struct App {
    cli: InputDebugCli,
    state: State,
}

impl Drop for App {
    fn drop(&mut self) {
        execute!(stdout(), DisableMouseCapture, LeaveAlternateScreen).ok();
        disable_raw_mode().ok();
    }
}

impl App {
    pub fn new(cli: InputDebugCli) -> Result<Self> {
        let (_w, h) = terminal::size()?;

        enable_raw_mode()?;
        execute!(stdout(), EnableMouseCapture, EnterAlternateScreen)?;

        Ok(Self {
            cli,
            state: State::new((h as usize).saturating_sub(3)),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let event_queue = EventQueue::with_ticks(100);
        let delay = Duration::from_millis(5);

        'main: loop {
            terminal.draw(|f| self.draw(f))?;

            for bf_event in event_queue.pop_all() {
                match &bf_event {
                    BfEvent::Tick => {
                        self.state.spinner_tick();
                    }
                    BfEvent::Input(event) => match event {
                        &Event::Key(key_event) => {
                            if key_event == KeyCode::Esc.into() {
                                break 'main;
                            }
                            self.state.input_history_add(bf_event.clone());
                        }
                        Event::Mouse(_) => {
                            if self.cli.enable_mouse {
                                self.state.input_history_add(bf_event);
                            }
                        }
                        &Event::Resize(_w, h) => {
                            let new_size = (h as usize).saturating_sub(3);
                            self.state.input_history_resize(new_size);
                        }
                        _ => {}
                    },
                }
            }

            thread::sleep(delay);
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout =
            Layout::vertical(vec![Constraint::Length(1), Constraint::Fill(1)])
                .split(frame.size());
        sublayouts!([header_area, content_area] = layout);

        self.draw_header(frame, header_area);
        self.draw_content(frame, content_area);
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::horizontal(vec![
            Constraint::Fill(1),   // Title
            Constraint::Length(1), // Spinner
        ])
        .spacing(1)
        .horizontal_margin(1)
        .split(area);
        sublayouts!([title_area, spinner_area] = layout);

        let title = Paragraph::new("Input Debugger (Press Esc to quit)");
        frame.render_widget(title, title_area);

        frame.render_widget(self.state.get_spinner(), spinner_area);
    }

    fn draw_content(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<_> = self
            .state
            .get_input_history()
            .map(|e| {
                Row::new(vec![
                    format!("{:0.6}", e.timestamp),
                    e.event.to_string(),
                ])
            })
            .collect();
        let table =
            Table::new(items, &[Constraint::Length(17), Constraint::Fill(1)])
                .block(Block::bordered())
                .column_spacing(2);
        frame.render_widget(table, area);
    }
}
