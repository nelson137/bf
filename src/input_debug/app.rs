use std::{
    io::{stdout, Write},
    thread,
    time::Duration,
};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph, Row, Table},
};

use crate::util::{
    err::BfResult,
    tui::{sublayouts, BfEvent, EventQueue, Frame, Terminal},
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
    pub fn new(cli: InputDebugCli) -> BfResult<Self> {
        let (_w, h) = terminal::size()?;

        enable_raw_mode()?;
        execute!(stdout(), EnableMouseCapture, EnterAlternateScreen)?;

        Ok(Self {
            cli,
            state: State::new((h as usize).saturating_sub(3)),
        })
    }

    pub fn run(&mut self) -> BfResult<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        let event_queue = EventQueue::new().with_tick_delay(100);
        let delay = Duration::from_millis(5);

        'main: loop {
            terminal.draw(|f| self.draw(f))?;

            for bf_event in event_queue.pop_all() {
                match bf_event {
                    BfEvent::Tick => {
                        self.state.spinner_inc();
                    }
                    BfEvent::Input(event) => match event {
                        Event::Key(key_event) => {
                            if key_event == KeyCode::Esc.into() {
                                break 'main;
                            }
                            self.state.input_history_add(bf_event);
                        }
                        Event::Mouse(_) => {
                            if self.cli.enable_mouse {
                                self.state.input_history_add(bf_event);
                            }
                        }
                        Event::Resize(_w, h) => {
                            let new_size = (h as usize).saturating_sub(3);
                            self.state.input_history_resize(new_size);
                        }
                    },
                }
            }

            thread::sleep(delay);
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
            .split(frame.size());
        sublayouts!([header_area, content_area] = layout);

        self.draw_header(frame, header_area);
        self.draw_content(frame, content_area);
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(1)
            //                Title               Spinner
            .constraints(vec![Constraint::Min(0), Constraint::Length(2)])
            .split(area);
        sublayouts!([title_area, spinner_area] = layout);

        let title = Paragraph::new("Input Debugger (Press Esc to quit)");
        frame.render_widget(title, title_area);

        let spinner = Paragraph::new(format!(" {}", self.state.get_spinner()));
        frame.render_widget(spinner, spinner_area);
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
        let table = Table::new(items)
            .block(Block::default().borders(Borders::ALL))
            .widths(&[Constraint::Length(17), Constraint::Min(0)])
            .column_spacing(2);
        frame.render_widget(table, area);
    }
}
