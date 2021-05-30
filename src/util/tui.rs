use std::{
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    io::Stdout,
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tui::{backend::CrosstermBackend, symbols, terminal};

type Backend = CrosstermBackend<Stdout>;
pub type Terminal = terminal::Terminal<Backend>;
pub type Frame<'a> = terminal::Frame<'a, Backend>;

pub trait KeyEventExt {
    fn is_alt(&self) -> bool;
    fn is_ctrl(&self) -> bool;
    fn is_ctrl_char(&self, c: char) -> bool;
    fn is_shift(&self) -> bool;
}

impl KeyEventExt for KeyEvent {
    fn is_alt(&self) -> bool {
        self.modifiers.intersects(KeyModifiers::ALT)
    }

    fn is_ctrl(&self) -> bool {
        self.modifiers.intersects(KeyModifiers::CONTROL)
    }

    fn is_ctrl_char(&self, c: char) -> bool {
        self.is_ctrl() && self.code == KeyCode::Char(c)
    }

    fn is_shift(&self) -> bool {
        self.modifiers.intersects(KeyModifiers::SHIFT)
            || match self.code {
                KeyCode::Char(c) if 'A' <= c && c <= 'Z' => true,
                _ => false,
            }
    }
}

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
                if key_event.is_ctrl() {
                    pieces.push(String::from("Ctrl"));
                }
                if key_event.is_alt() {
                    pieces.push(String::from("Alt"));
                }
                if key_event.is_shift() {
                    pieces.push(String::from("Shift"));
                }
                pieces.push(match key_event.code {
                    KeyCode::Char('\'') => String::from("'\\''"),
                    KeyCode::Char(c) => format!("'{}'", c),
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
where
    Func: FnOnce(MutexGuard<T>) -> Ret,
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
                if let Ok(evt) = event::read() {
                    mutex_safe_do(&*data, |mut q| {
                        q.push_back(BfEvent::Input(evt))
                    });
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

    pub fn pop_all(&self) -> Vec<BfEvent> {
        mutex_safe_do(&*self.data, |mut q| q.drain(..).collect())
    }
}

pub const TAPE_BORDER_SET: symbols::line::Set = symbols::line::NORMAL;

pub struct TapeBorderHorizontal {
    left_capped: &'static str,
    left_uncapped: &'static str,
    middle: &'static str,
    right_capped: &'static str,
    right_uncapped: &'static str,
}

impl TapeBorderHorizontal {
    pub fn left(&self, capped: bool) -> &'static str {
        if capped {
            self.left_capped
        } else {
            self.left_uncapped
        }
    }

    pub fn middle(&self) -> &'static str {
        self.middle
    }

    pub fn right(&self, capped: bool) -> &'static str {
        if capped {
            self.right_capped
        } else {
            self.right_uncapped
        }
    }
}

pub trait LineSymbolsExt {
    fn top(&self) -> TapeBorderHorizontal;
    fn bottom(&self) -> TapeBorderHorizontal;
}

impl LineSymbolsExt for symbols::line::Set {
    fn top(&self) -> TapeBorderHorizontal {
        TapeBorderHorizontal {
            left_capped: self.top_left,
            left_uncapped: self.horizontal_down,
            middle: self.horizontal,
            right_capped: self.top_right,
            right_uncapped: self.horizontal_down,
        }
    }

    fn bottom(&self) -> TapeBorderHorizontal {
        TapeBorderHorizontal {
            left_capped: self.bottom_left,
            left_uncapped: self.horizontal_up,
            middle: self.horizontal,
            right_capped: self.bottom_right,
            right_uncapped: self.horizontal_up,
        }
    }
}