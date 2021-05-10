use std::{
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use crossterm::{
    event::{Event, KeyCode, KeyModifiers, read},
};

pub enum BfEvent {
    Tick,
    Input(Event),
}

impl Display for BfEvent {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        match self {
            Self::Tick => fmt.write_str("Tick"),
            Self::Input(Event::Key(key_event)) => {
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
                fmt.write_str(&pieces.join(" + "))
            }
            Self::Input(event) => fmt.write_str(&format!("{:?}", event)),
        }
    }
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

#[derive(Clone)]
pub struct EventQueue {
    data: Arc<Mutex<VecDeque<BfEvent>>>,
}

impl EventQueue {
    pub fn new() -> Self {
        let data = Arc::new(Mutex::new(VecDeque::new()));

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

    pub fn with_tick_delay(self, tick_delay: u64) -> Self {
        let _tick_thread = {
            let data = self.data.clone();
            thread::spawn(move || loop {
                mutex_safe_do(&*data, |mut q| q.push_back(BfEvent::Tick));
                thread::sleep(Duration::from_millis(tick_delay));
            })
        };
        self
    }

    pub fn pop_event(&self) -> Option<BfEvent> {
        mutex_safe_do(&*self.data, |mut q| q.pop_front())
    }
}
