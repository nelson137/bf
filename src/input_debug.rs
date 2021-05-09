use std::{
    collections::VecDeque,
    io::{self, Write},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::{
    event::{EnableMouseCapture, Event, EventStream, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode, size},
};
use futures::{StreamExt, executor::block_on, future::FutureExt, select};
use futures_timer::Delay;
use structopt::StructOpt;
use tui::{
    Frame,
    backend::{Backend, CrosstermBackend}, Terminal,
    layout::{Constraint, Direction, Layout},
    widgets::{Borders, Block, Row, Table},
};

use crate::{
    subcmd::SubCmd,
    util::die,
};

const ABOUT: &str = "Live scripting playground";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct InputDebugCli {
}

struct InputHistoryEntry {
    event: Event,
    timestamp: f64,
}

const SPINNER: &str = "|/-\\";

struct State {
    spinner_i: usize,
    input_history_size: usize,
    input_history: VecDeque<InputHistoryEntry>,
}

impl State {
    pub fn new(input_history_size: usize) -> Self {
        Self {
            spinner_i: 0,
            input_history_size,
            input_history: VecDeque::with_capacity(input_history_size),
        }
    }

    pub fn input_history_resize(&mut self, size: usize) {
        self.input_history_size = size;
        while self.input_history.len() > self.input_history_size {
            self.input_history.pop_back();
        }
    }
}

fn draw<B: Backend>(frame: &mut Frame<B>, state: &State) {
    let width = frame.size().width;
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(1),
            Constraint::Min(0)
        ])
        .split(frame.size());

    let spinner_char = SPINNER.chars().nth(state.spinner_i)
        .expect("Invalid spinner char index");
    let title_table_items = vec![Row::new(vec![
        String::from(" Input Debugger"),
        format!("{} ", spinner_char),
    ])];
    let title_constraints = [
        // -3 for: the default col space (-1) and the spinner (-2)
        Constraint::Length(width-3),
        Constraint::Length(3),
    ];
    let title_table = Table::new(title_table_items)
        .block(Block::default())
        .widths(&title_constraints);
    frame.render_widget(title_table, sections[0]);

    let table_block = Block::default().borders(Borders::ALL);
    let items: Vec<_> = state.input_history.iter()
        .map(|e| Row::new(vec![
            format!("{:0.6}", e.timestamp),
            format!("{:?}", e.event),
        ]))
        .collect();
    let table = Table::new(items)
        .block(table_block)
        .widths(&[
            Constraint::Length(17),
            Constraint::Min(0),
        ])
        .column_spacing(2);
    frame.render_widget(table, sections[1]);
}

async fn run() {
    let mut terminal =
        Terminal::new(CrosstermBackend::new(io::stdout()))
        .unwrap_or_else(|e| die(e.to_string()));

    let (_w, h) = size().unwrap_or_else(|e| die(e.to_string())).into();

    let mut reader = EventStream::new();
    let mut state = State::new((h as usize).saturating_sub(3));

    loop {
        terminal.draw(|f| draw(f, &state))
            .unwrap_or_else(|e| die(e.to_string()));

        let mut delay_async = Delay::new(Duration::from_millis(100)).fuse();
        let mut event_async = reader.next().fuse();

        select! {
            _ = delay_async => state.spinner_i = (state.spinner_i+1) % 4,
            some_event = event_async => match some_event {
                None => break,
                Some(Err(err)) => die(err.to_string()),
                Some(Ok(event)) => match event {
                    Event::Key(key_event) => {
                        if key_event == KeyCode::Esc.into() {
                            break;
                        }
                        if state.input_history.len() >= 10 {
                            state.input_history.pop_back();
                        }
                        state.input_history.push_front(InputHistoryEntry {
                            event,
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .expect("Time went backwards")
                                .as_secs_f64(),
                        });
                    }
                    Event::Resize(_w, h) => {
                        let new_size = (h as usize).saturating_sub(3);
                        state.input_history_resize(new_size);
                    }
                    _ => {}
                },
            },
        };
    }
}

impl SubCmd for InputDebugCli {
    fn run(self) {
        enable_raw_mode().unwrap_or_else(|e| die(e.to_string()));
        let mut stdout = io::stdout();
        execute!(stdout, EnableMouseCapture, EnterAlternateScreen)
            .unwrap_or_else(|e| die(e.to_string()));

        block_on(run());
        disable_raw_mode().unwrap_or_else(|e| die(e.to_string()));
    }
}
