use std::{
    io,
    thread,
    time::Duration,
};

use crossterm::{
    event::{Event, KeyCode},
    terminal::size,
};
use tui::{
    Frame,
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Row, Table},
};

use crate::{
    tui_util::{BfEvent, EventQueue},
    util::die,
};

use super::state::State;

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
        // -3 for the default col space (-1) and the spinner (-2)
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

pub fn run() {
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
                        state.input_history_add(bf_event);
                    }
                    Event::Mouse(_) => state.input_history_add(bf_event),
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
