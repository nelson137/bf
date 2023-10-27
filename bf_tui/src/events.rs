use std::{
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    sync::{Arc, Mutex, MutexGuard},
    thread,
    time::Duration,
};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
};

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

#[derive(Clone)]
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
                    KeyCode::Char(c) => format!("'{c}'"),
                    KeyCode::F(f) => format!("F{f}"),
                    keycode => format!("{keycode:?}"),
                });
                fmt.write_str(&pieces.join(" + "))
            }
            Self::Input(event) => fmt.write_str(&format!("{event:?}")),
        }
    }
}

type EventQueueBuf = VecDeque<BfEvent>;

#[derive(Clone)]
pub struct EventQueue {
    data: Arc<Mutex<EventQueueBuf>>,
}

impl EventQueue {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let this = Self {
            data: Arc::default(),
        };

        {
            let this = this.clone();
            thread::spawn(move || loop {
                if let Ok(evt) = event::read() {
                    this.safe_mutate(|mut q| q.push_back(BfEvent::Input(evt)));
                }
                thread::yield_now();
            });
        }

        this
    }

    fn safe_mutate<Ret>(
        &self,
        func: impl FnOnce(MutexGuard<EventQueueBuf>) -> Ret,
    ) -> Ret {
        func(self.data.lock().expect("EventQueue mutex is poisoned"))
    }

    pub fn with_ticks(delay_ms: u64) -> Self {
        let this = Self::new();

        {
            let this = this.clone();
            thread::spawn(move || loop {
                this.safe_mutate(|mut q| q.push_back(BfEvent::Tick));
                thread::sleep(Duration::from_millis(delay_ms));
            });
        }

        this
    }

    pub fn pop_all(&self) -> VecDeque<BfEvent> {
        let mut events: Vec<BfEvent> =
            self.safe_mutate(|mut q| q.drain(..).collect());
        if events.is_empty() {
            return events.into();
        }

        macro_rules! bfevent_char_matcher {
            ($c:expr) => {
                BfEvent::Input(Event::Key(KeyEvent {
                    code: KeyCode::Char($c),
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::NONE,
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
