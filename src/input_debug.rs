use std::{
    collections::vec_deque::{self, VecDeque},
    io::{self, Write},
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::{
    event::{
        DisableMouseCapture,
        EnableMouseCapture,
        Event,
        KeyCode,
        KeyModifiers,
        read
    },
    execute,
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
        disable_raw_mode,
        enable_raw_mode,
        size
    },
};
use structopt::StructOpt;
use tui::{
    Frame,
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Row, Table},
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

trait PrintableEvent {
    fn to_string(&self) -> String;
}

impl PrintableEvent for Event {
    fn to_string(&self) -> String {
        match self {
            Self::Key(key_event) => {
                let mut pieces: Vec<String> = Vec::with_capacity(4);
                if key_event.modifiers.intersects(KeyModifiers::CONTROL) {
                    pieces.push(String::from("Ctrl"));
                }
                if key_event.modifiers.intersects(KeyModifiers::ALT) {
                    pieces.push(String::from("Alt"));
                }
                if key_event.modifiers.intersects(KeyModifiers::SHIFT) {
                    pieces.push(String::from("Shift"));
                }
                pieces.push(match key_event.code {
                    KeyCode::Char('\'') => String::from("'\\''"),
                    KeyCode::Char(c) => format!("'{}'", c.to_lowercase()),
                    KeyCode::F(f) => format!("F{}", f),
                    keycode => format!("{:?}", keycode),
                });
                pieces.join(" + ")
            }
            event => format!("{:?}", event)
        }
    }
}

enum BfEvent {
    Tick,
    Input(Event),
}

#[derive(Clone)]
struct EventQueue {
    data: Arc<Mutex<VecDeque<BfEvent>>>
}

fn mutex_safe_do<T, Ret, Func>(data: &Mutex<T>, func: Func) -> Ret
    where Func: FnOnce(MutexGuard<T>) -> Ret
{
    if let Ok(queue) = data.lock() {
        func(queue)
    } else {
        panic!("EventQueue: failed because of poisoned mutex");
    }
}

impl EventQueue {
    pub fn with_tick_delay(tick_delay: u64) -> Self {
        let data = Arc::new(Mutex::new(VecDeque::new()));

        let _tick_thread = {
            let data = data.clone();
            thread::spawn(move || loop {
                mutex_safe_do(&*data, |mut q| q.push_back(BfEvent::Tick));
                thread::sleep(Duration::from_millis(tick_delay));
            })
        };

        let _input_thread = {
            let data = data.clone();
            thread::spawn(move || loop {
                if let Ok(evt) = read() {
                    mutex_safe_do(
                        &*data,
                        |mut q| q.push_back(BfEvent::Input(evt))
                    );
                }
                thread::yield_now();
            })
        };

        Self { data }
    }

    pub fn pop_event(&self) -> Option<BfEvent> {
        mutex_safe_do(&*self.data, |mut q| q.pop_front())
    }
}

struct InputHistoryEntry {
    event: Event,
    timestamp: f64,
}

impl InputHistoryEntry {
    pub fn new(event: Event) -> Self {
        Self {
            event,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64(),
        }
    }
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

    pub fn get_spinner(&self) -> char {
        SPINNER.chars().nth(self.spinner_i)
            .expect("Invalid spinner char index")
    }

    pub fn get_input_history(&self) -> vec_deque::Iter<InputHistoryEntry> {
        self.input_history.iter()
    }

    pub fn spinner_inc(&mut self) {
        self.spinner_i = (self.spinner_i + 1) % 4;
    }

    pub fn input_history_resize(&mut self, size: usize) {
        self.input_history_size = size;
        while self.input_history.len() > self.input_history_size {
            self.input_history.pop_back();
        }
    }

    pub fn input_history_add(&mut self, event: Event) {
        if self.input_history.len() >= self.input_history_size {
            self.input_history.pop_back();
        }
        self.input_history.push_front(InputHistoryEntry::new(event));
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

    let title_table_items = vec![Row::new(vec![
        String::from(" Input Debugger (Press Esc to quit)"),
        format!("{} ", state.get_spinner()),
    ])];
    let title_constraints = [
        // -3 for: the default col space (-1) and the spinner (-2)
        Constraint::Length(width - 3),
        Constraint::Length(3),
    ];
    let title_table = Table::new(title_table_items)
        .block(Block::default())
        .widths(&title_constraints);
    frame.render_widget(title_table, sections[0]);

    let table_block = Block::default().borders(Borders::ALL);
    let items: Vec<_> = state.get_input_history()
        .map(|e| Row::new(vec![
            format!("{:0.6}", e.timestamp),
            e.event.to_string(),
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

fn run() {
    let mut terminal =
        Terminal::new(CrosstermBackend::new(io::stdout()))
        .unwrap_or_else(|e| die(e.to_string()));

    let (_w, h) = size().unwrap_or_else(|e| die(e.to_string())).into();

    let event_queue = EventQueue::with_tick_delay(100);
    let mut state = State::new((h as usize).saturating_sub(3));
    let delay = Duration::from_millis(5);

    'main: loop {
        terminal.draw(|f| draw(f, &state))
            .unwrap_or_else(|e| die(e.to_string()));

        while let Some(bf_event) = event_queue.pop_event() {
            match bf_event {
                BfEvent::Tick => state.spinner_inc(),
                BfEvent::Input(event) => match event {
                    Event::Key(key_event) => {
                        if key_event == KeyCode::Esc.into() {
                            break 'main;
                        }
                        state.input_history_add(event);
                    }
                    Event::Mouse(_) => state.input_history_add(event),
                    Event::Resize(_w, h) => {
                        let new_size = (h as usize).saturating_sub(3);
                        state.input_history_resize(new_size);
                    }
                }
            }
        }

        thread::sleep(delay);
    }
}

impl SubCmd for InputDebugCli {
    fn run(self) {
        enable_raw_mode().unwrap_or_else(|e| die(e.to_string()));
        execute!(io::stdout(), EnableMouseCapture, EnterAlternateScreen)
            .unwrap_or_else(|e| die(e.to_string()));

        run();

        execute!(io::stdout(), DisableMouseCapture, LeaveAlternateScreen)
            .unwrap_or_else(|e| die(e.to_string()));
        disable_raw_mode().unwrap_or_else(|e| die(e.to_string()));
    }
}
