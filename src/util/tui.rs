use std::{
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    io::Stdout,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::CrosstermBackend, symbols::line, terminal};

use super::common::mutex_safe_do;

type Backend = CrosstermBackend<Stdout>;
pub type Terminal = terminal::Terminal<Backend>;
pub type Frame<'a> = terminal::Frame<'a, Backend>;

#[macro_export]
macro_rules! sublayouts {
    ([$($binding:tt),*] = $layout:tt) => {
        let mut _index = 0usize..;
        $(
            let $binding = $layout[_index.next().unwrap()];
        )*
    };
}
pub use sublayouts;

pub const TAPE_BORDER_SET: line::Set = line::NORMAL;

pub const TAPE_HORIZONTAL_BORDER_TOP: TapeBorderHorizontal =
    TapeBorderHorizontal {
        left_capped: TAPE_BORDER_SET.top_left,
        left_uncapped: TAPE_BORDER_SET.horizontal_down,
        middle: TAPE_BORDER_SET.horizontal,
        right_capped: TAPE_BORDER_SET.top_right,
        right_uncapped: TAPE_BORDER_SET.horizontal_down,
    };

pub const TAPE_HORIZONTAL_BORDER_BOTTOM: TapeBorderHorizontal =
    TapeBorderHorizontal {
        left_capped: TAPE_BORDER_SET.bottom_left,
        left_uncapped: TAPE_BORDER_SET.horizontal_up,
        middle: TAPE_BORDER_SET.horizontal,
        right_capped: TAPE_BORDER_SET.bottom_right,
        right_uncapped: TAPE_BORDER_SET.horizontal_up,
    };

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
            || matches!(self.code, KeyCode::Char('A'..='Z'))
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

    pub fn with_ticks(delay_ms: u64) -> Self {
        let this = Self::new();

        let _tick_thread = {
            let data = this.data.clone();
            thread::spawn(move || loop {
                mutex_safe_do(&*data, |mut q| q.push_back(BfEvent::Tick));
                thread::sleep(Duration::from_millis(delay_ms));
            })
        };

        this
    }

    pub fn pop_all(&self) -> VecDeque<BfEvent> {
        let mut events: Vec<BfEvent> =
            mutex_safe_do(&*self.data, |mut q| q.drain(..).collect());
        if events.is_empty() {
            return events.into();
        }

        macro_rules! bfevent_char_matcher {
            ($c:expr) => {
                BfEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char($c),
                    modifiers: KeyModifiers::NONE,
                }))
            };
        }

        macro_rules! insert_keycode {
            ($code:ident, $i:ident) => {
                events[$i - 2] =
                    BfEvent::Input(Event::Key(KeyCode::$code.into()));
                events.remove($i);
                events.remove($i - 1);
                $i -= 2;
            };
        }

        let mut i = events.len();
        while i >= 3 {
            i -= 1;
            if let (
                bfevent_char_matcher!('~'),
                BfEvent::Input(Event::Key(k_evt)),
                bfevent_char_matcher!('['),
            ) = (&events[i], &events[i - 1], &events[i - 2])
            {
                if k_evt.code == KeyCode::Char('5') {
                    insert_keycode!(PageUp, i);
                } else if k_evt.code == KeyCode::Char('6') {
                    insert_keycode!(PageDown, i);
                }
            }
        }

        events.into()
    }
}

pub trait LineSetExts {
    fn top_divider(&self, width: usize, label: &str) -> String;
    fn middle_divider(&self, width: usize, label: &str) -> String;
    fn bottom_divider(&self, width: usize) -> String;
}

impl LineSetExts for line::Set {
    fn top_divider(&self, width: usize, label: &str) -> String {
        self.top_left.to_owned()
            + label
            + &self
                .horizontal
                .repeat(width.saturating_sub(2 + label.len()))
            + self.top_right
    }

    fn middle_divider(&self, width: usize, label: &str) -> String {
        self.vertical_right.to_owned()
            + label
            + &self
                .horizontal
                .repeat(width.saturating_sub(2 + label.len()))
            + self.vertical_left
    }

    fn bottom_divider(&self, width: usize) -> String {
        self.bottom_left.to_owned()
            + &self.horizontal.repeat(width.saturating_sub(2))
            + self.bottom_right
    }
}
