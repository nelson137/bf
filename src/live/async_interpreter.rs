use std::{
    sync::{Arc, Barrier},
    thread,
};

use crate::{
    interpreter::{Interpreter, Tape},
    util::{
        err::BfResult,
        sync::{SharedBool, SharedCell},
    },
};

#[derive(Clone)]
pub enum Status {
    Running,
    Done,
    Error(String),
    FatalError(String),
}

impl Default for Status {
    fn default() -> Self {
        Self::Done
    }
}

#[derive(Clone, Default)]
pub struct State {
    pub status: Status,
    pub tape: Tape,
}

#[derive(Clone)]
pub struct AsyncInterpreter {
    stop: SharedBool,
    restart_barrier: Arc<Barrier>,
    program: SharedCell<(String, String)>,
    state: SharedCell<State>,
}

const ERROR_POISONED: &'static str =
    "an interpreter thread mutex was poisoned";

impl AsyncInterpreter {
    pub fn new(code: String, input: String) -> Self {
        let this = Self {
            stop: SharedBool::new(false),
            restart_barrier: Arc::new(Barrier::new(2)),
            program: SharedCell::new((code, input)),
            state: SharedCell::default(),
        };

        let shared = this.clone();
        thread::spawn(move || loop {
            let mut int = match shared.program.load() {
                Ok((code, input)) => Interpreter::new(code, input),
                Err(_) => {
                    thread::yield_now();
                    shared.restart_barrier.wait();
                    continue;
                }
            };

            let set_state = |status: Status, int: &Interpreter| {
                shared
                    .state
                    .store(State {
                        status,
                        tape: int.tape.clone(),
                    })
                    .ok()
            };

            shared.stop.store(false);

            while !shared.stop.load() {
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

    pub fn restart(&self, code: String, input: String) -> BfResult<()> {
        self.stop.store(true);
        self.program
            .store((code, input))
            .or(Err(ERROR_POISONED.clone()))?;
        self.restart_barrier.wait();
        self.stop.store(false);
        Ok(())
    }

    pub fn state(&self) -> State {
        match self.state.load() {
            Ok(state) => state,
            Err(()) => State {
                status: Status::FatalError(ERROR_POISONED.into()),
                tape: Tape::default(),
            },
        }
    }
}
