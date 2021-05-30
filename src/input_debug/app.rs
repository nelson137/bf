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
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Row, Table},
};

use crate::util::{
    err::BfResult,
    tui::{BfEvent, EventQueue, Terminal},
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
            self.draw(&mut terminal)?;

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

    fn draw(&self, terminal: &mut Terminal) -> BfResult<()> {
        Ok(terminal
            .draw(|frame| {
                let width = frame.size().width;
                let sections = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(frame.size());

                let title_table_items = vec![Row::new(vec![
                    String::from(" Input Debugger (Press Esc to quit)"),
                    format!("{} ", self.state.get_spinner()),
                ])];
                let title_constraints = [
                    // -3 for the default col space (-1) and the spinner (-2)
                    Constraint::Length(width - 3),
                    Constraint::Length(3),
                ];
                let title_table = Table::new(title_table_items)
                    .block(Block::default())
                    .widths(&title_constraints);
                frame.render_widget(title_table, sections[0]);

                let table_block = Block::default().borders(Borders::ALL);
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
                    .block(table_block)
                    .widths(&[Constraint::Length(17), Constraint::Min(0)])
                    .column_spacing(2);
                frame.render_widget(table, sections[1]);
            })
            .and(Ok(()))?)
    }
}
