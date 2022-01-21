use std::{
    collections::{vec_deque, VecDeque},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::util::tui::{BfEvent, Spinner};

pub struct InputHistoryEntry {
    pub event: BfEvent,
    pub timestamp: f64,
}

impl InputHistoryEntry {
    pub fn new(event: BfEvent) -> Self {
        Self {
            event,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64(),
        }
    }
}

pub struct State {
    spinner: Spinner,
    input_history_size: usize,
    input_history: VecDeque<InputHistoryEntry>,
}

impl State {
    pub fn new(input_history_size: usize) -> Self {
        Self {
            spinner: Spinner::default(),
            input_history_size,
            input_history: VecDeque::with_capacity(input_history_size),
        }
    }

    pub fn get_input_history(&self) -> vec_deque::Iter<InputHistoryEntry> {
        self.input_history.iter()
    }

    pub fn get_spinner(&self) -> Spinner {
        self.spinner
    }

    pub fn spinner_tick(&mut self) {
        self.spinner.tick();
    }

    pub fn input_history_resize(&mut self, size: usize) {
        self.input_history_size = size;
        while self.input_history.len() > self.input_history_size {
            self.input_history.pop_front();
        }
    }

    pub fn input_history_add(&mut self, event: BfEvent) {
        if self.input_history.len() >= self.input_history_size {
            self.input_history.pop_front();
        }
        self.input_history.push_back(InputHistoryEntry::new(event));
    }
}
