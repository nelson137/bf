use std::{
    fs::File,
    io::{stdout, Write},
    path::PathBuf,
};

use anyhow::{Context, Error, Result};
use bf::interpreter::Interpreter;
use bf_tui::{
    events::{BfEvent, EventQueue, KeyEventExt},
    widgets::run::{AppWidget, AppWidgetState},
    Terminal,
};
use crossterm::{
    event::{Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::CrosstermBackend;

use crate::{err_file_open, err_file_write, utils::read::read_script};

fn reset_terminal() {
    disable_raw_mode().ok();
}

fn set_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        reset_terminal();
        original_hook(panic_info);
    }));
}

pub struct App {
    event_queue: EventQueue,
    show_tape: bool,
    ascii_values: bool,
    outfile: Option<PathBuf>,
    interpreter: Interpreter,
    render_state: AppWidgetState,
}

impl Drop for App {
    fn drop(&mut self) {
        reset_terminal();
    }
}

impl App {
    pub fn new(cli: super::RunCli) -> Result<Self> {
        set_panic_hook();
        enable_raw_mode()?;

        let script_lines = read_script(cli.infile.as_ref())?;
        let script = script_lines.iter().flat_map(|l| l.as_bytes()).copied();

        let input = cli.input.into_bytes().into_iter().collect();

        Ok(Self {
            event_queue: EventQueue::with_ticks(cli.delay),
            show_tape: cli.show_tape,
            ascii_values: cli.ascii_values,
            outfile: cli.outfile,
            interpreter: Interpreter::new(script, input, None),
            render_state: AppWidgetState::default(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.run_script()?;

        if let Some(path) = &self.outfile {
            File::create(path)
                .with_context(|| err_file_open!(path))?
                .write_all(self.interpreter.output_bytes())
                .with_context(|| err_file_write!(path))?;
        }

        Ok(())
    }

    fn run_script(&mut self) -> Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;

        let script_err = self.run_script_loop(&mut terminal)?;

        reset_terminal();
        terminal.set_cursor(0, self.render_state.height).ok();

        if let Some(err) = script_err {
            eprintln!();
            eprintln!("Error: {err}");
        }

        Ok(())
    }

    fn run_script_loop(
        &mut self,
        terminal: &mut Terminal,
    ) -> Result<Option<Error>> {
        self.draw_frame(terminal)?;

        'mainloop: loop {
            for event in self.event_queue.pop_all() {
                match event {
                    BfEvent::Tick => match self.interpreter.next() {
                        Some(frame) => match frame {
                            Ok(_) => self.draw_frame(terminal)?,
                            Err(err) => break 'mainloop Ok(Some(err)),
                        },
                        None => break 'mainloop Ok(None),
                    },
                    BfEvent::Input(input_event) => match input_event {
                        Event::Key(e)
                            if e.is_ctrl() && e.code == KeyCode::Char('c') =>
                        {
                            break 'mainloop Ok(None)
                        }
                        _ => (),
                    },
                };
            }
        }
    }

    fn draw_frame(&mut self, terminal: &mut Terminal) -> Result<()> {
        terminal.draw(|f| {
            let area = f.size();
            let widget = AppWidget {
                show_tape: self.show_tape,
                ascii_values: self.ascii_values,
                interpreter: &self.interpreter,
            };
            f.render_stateful_widget(widget, area, &mut self.render_state);
        })?;
        Ok(())
    }
}
