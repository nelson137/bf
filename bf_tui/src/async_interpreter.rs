use std::{
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    sync::{Arc, Barrier},
    thread,
};

use anyhow::{bail, Result};
use bf::interpreter::{Interpreter, Tape};
use bf_utils::sync::{SharedBool, SharedCell};

#[derive(Clone, Eq, PartialEq)]
pub enum Status {
    Running,
    WaitingForInput,
    Done,
    Error(String),
    FatalError(String),
}

impl Default for Status {
    fn default() -> Self {
        Self::Done
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Running => write!(f, "Running…"),
            Self::WaitingForInput => write!(f, "Waiting for Input…"),
            Self::Done => write!(f, "Done"),
            Self::Error(_) | Self::FatalError(_) => write!(f, "ERROR"),
        }
    }
}

#[derive(Clone, Default)]
pub struct State {
    pub status: Status,
    pub tape: Tape,
    pub output: Vec<u8>,
}

#[derive(Clone)]
pub struct AsyncInterpreter {
    stop: SharedBool,
    restart_barrier: Arc<Barrier>,
    program: SharedCell<(Vec<u8>, VecDeque<u8>, Option<u8>)>,
    state: SharedCell<State>,
}

const ERROR_POISONED: &str = "an interpreter thread mutex was poisoned";

impl AsyncInterpreter {
    pub fn new(
        code: Vec<u8>,
        input: VecDeque<u8>,
        auto_input: Option<u8>,
    ) -> Self {
        let this = Self {
            stop: SharedBool::new(false),
            restart_barrier: Arc::new(Barrier::new(2)),
            program: SharedCell::new((code, input, auto_input)),
            state: SharedCell::default(),
        };

        let shared = this.clone();
        thread::spawn(move || loop {
            let mut int = if let Some((code, input, auto_input)) =
                shared.program.load()
            {
                Interpreter::new(code.into_iter(), input, auto_input)
            } else {
                thread::yield_now();
                shared.restart_barrier.wait();
                continue;
            };

            let set_state = |status: Status, int: &Interpreter| {
                shared.state.store(State {
                    status,
                    tape: int.tape.clone(),
                    output: int.output.clone(),
                });
            };

            shared.stop.store(false);

            while !shared.stop.load() {
                match int.peek() {
                    Some(',') if int.input.is_empty() => {
                        set_state(Status::WaitingForInput, &int);
                    }
                    _ => (),
                }
                match int.next() {
                    None => {
                        set_state(Status::Done, &int);
                        break;
                    }
                    Some(Err(err)) => {
                        set_state(Status::Error(err.to_string()), &int);
                        break;
                    }
                    Some(Ok(_)) => {
                        set_state(Status::Running, &int);
                    }
                }
            }

            thread::yield_now();
            shared.restart_barrier.wait();
        });

        this
    }

    pub fn restart(
        &self,
        code: Vec<u8>,
        input: VecDeque<u8>,
        auto_input: Option<u8>,
    ) -> Result<()> {
        self.stop.store(true);
        if !self.program.store((code, input, auto_input)) {
            bail!(ERROR_POISONED);
        }
        self.restart_barrier.wait();
        self.stop.store(false);
        Ok(())
    }

    pub fn state(&self) -> State {
        match self.state.load() {
            Some(state) => state,
            None => State {
                status: Status::FatalError(ERROR_POISONED.into()),
                tape: Tape::default(),
                output: Vec::new(),
            },
        }
    }
}
