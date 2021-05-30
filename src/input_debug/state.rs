use std::{collections::{VecDeque, vec_deque}, time::{SystemTime, UNIX_EPOCH}};

use crate::util::tui::BfEvent;

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

const SPINNER: &str = "|/-\\";

pub struct State {
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

    pub fn input_history_add(&mut self, event: BfEvent) {
        if self.input_history.len() >= self.input_history_size {
            self.input_history.pop_back();
        }
        self.input_history.push_front(InputHistoryEntry::new(event));
    }
}
