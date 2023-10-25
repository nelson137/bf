use std::{
    fs::File,
    io::{stderr, stdout, Write},
    thread,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{
    backend::CrosstermBackend,
    style::{Color, Style},
};
use ratatui_textarea::TextArea;
use tracing::trace;

use crate::{
    util::{
        common::{sha1_digest, Sha1Digest},
        read::read_script_file,
        tui::{BfEvent, EventQueue, KeyEventExt, Terminal},
    },
    widgets::Spinner,
};

use super::{
    async_interpreter::{AsyncInterpreter, Status},
    cli::LiveCli,
    logging::init_logging,
    textarea::TextAreaExts,
    widgets::{AppWidget, Dialogue, DialogueCommand, TapeViewportState},
};

fn reset_terminal() {
    execute!(stderr(), LeaveAlternateScreen).ok();
    execute!(stdout(), DisableMouseCapture, LeaveAlternateScreen).ok();
    disable_raw_mode().ok();
}

fn set_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        reset_terminal();
        original_hook(panic_info);
    }));
}

pub struct App<'code, 'dialogue> {
    term_width: usize,
    term_height: usize,
    file_path: Option<String>,
    should_quit: bool,
    spinner: Spinner,
    code: TextArea<'code>,
    tape_viewport: TapeViewportState,
    input: String,
    auto_input: Option<u8>,
    clean_hash: Sha1Digest,
    event_queue: EventQueue,
    delay: Duration,
    dialogue: Option<Box<Dialogue<'dialogue>>>,
    async_interpreter: AsyncInterpreter,
}

impl Drop for App<'_, '_> {
    fn drop(&mut self) {
        reset_terminal();
    }
}

impl<'code, 'dialogue> App<'code, 'dialogue> {
    pub fn new(cli: LiveCli) -> Result<Self> {
        init_logging()?;
        trace!("initialize TUI app");

        set_panic_hook();

        enable_raw_mode()?;
        execute!(stdout(), EnableMouseCapture, EnterAlternateScreen)?;

        let script = if let Some(path) = &cli.infile {
            read_script_file(path)?
        } else {
            Vec::new()
        };

        let script_raw = script
            .iter()
            .flat_map(|l| l.as_bytes())
            .copied()
            .collect::<Vec<_>>();

        let mut code = TextArea::from(script);
        code.set_line_number_style(Style::default().fg(Color::Yellow));
        code.set_cursor_line_style(Style::default());

        let interpreter_code = code.bytes().collect();

        Ok(Self {
            term_width: 0,
            term_height: 0,
            file_path: cli.infile.map(|p| p.to_string_lossy().into()),
            should_quit: false,
            spinner: Spinner::default(),
            code,
            tape_viewport: TapeViewportState::new(cli.ascii_values),
            input: String::new(),
            auto_input: None,
            clean_hash: sha1_digest(script_raw),
            event_queue: EventQueue::with_ticks(100),
            delay: Duration::from_millis(20),
            dialogue: None,
            async_interpreter: AsyncInterpreter::new(
                interpreter_code,
                Default::default(),
                None,
            ),
        })
    }

    fn get_file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    fn is_dirty(&self) -> bool {
        self.code.hash() != self.clean_hash
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let mut restart_interpreter: bool;

        let term_size = terminal.size()?;
        self.term_width = term_size.width as usize;
        self.term_height = term_size.height as usize;

        while !self.should_quit {
            restart_interpreter = false;

            self.draw(&mut terminal)?;

            for event in self.event_queue.pop_all() {
                match event {
                    BfEvent::Tick => self.spinner.tick(),
                    BfEvent::Input(input_event) => match input_event {
                        Event::Key(e) => {
                            restart_interpreter = self.handle_key_event(e)
                        }
                        Event::Resize(width, height) => {
                            self.term_width = width as usize;
                            self.term_height = height as usize;
                        }
                        _ => (),
                    },
                };
            }

            if restart_interpreter {
                let status = self.async_interpreter.state().status;
                if let Status::FatalError(fe) = status {
                    self.dialogue = Some(Box::new(Dialogue::error(fe)));
                }
                self.async_interpreter.restart(
                    self.code.bytes().collect(),
                    self.input.bytes().collect(),
                    self.auto_input,
                )?;
            }

            thread::yield_now();
            thread::sleep(self.delay);
        }

        Ok(())
    }

    fn draw(&self, terminal: &mut Terminal) -> Result<()> {
        let widget = AppWidget {
            is_dirty: self.is_dirty(),
            async_interpreter: self.async_interpreter.state(),
            editor: self.code.widget(),
            dialogue: self.dialogue.as_deref(),
            file_path: self.file_path.as_deref(),
            spinner: self.spinner,
            tape_viewport: self.tape_viewport,
            term_height: self.term_height,
            term_width: self.term_width,
        };
        terminal.draw(|f| f.render_widget(widget, f.size()))?;
        Ok(())
    }

    fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        let mut restart_interpreter = false;

        if let Some(dialogue) = &mut self.dialogue {
            match dialogue.on_event(event) {
                DialogueCommand::None => {}
                DialogueCommand::Dismissed => {
                    self.dialogue = None;
                }
                DialogueCommand::ConfirmUnsavedChangesConfirmed => {
                    self.should_quit = true;
                }
                DialogueCommand::FileSaveAsSubmitted(path) => {
                    self.file_path = Some(path);
                    self.dialogue = None;
                    self.on_save();
                }
                DialogueCommand::ScriptInputSubmitted(input) => {
                    self.input = input;
                    self.dialogue = None;
                    restart_interpreter = true;
                }
                DialogueCommand::ScriptAutoInputSubmitted(input) => {
                    self.auto_input = input;
                    self.dialogue = None;
                    restart_interpreter = true;
                }
            }
        } else {
            self.code.on_event_multi_line(event);
            match event.code {
                KeyCode::Char(c) if event.is_ctrl() => match c {
                    's' => self.on_save(),
                    'x' => self.on_save_as(),
                    'a' => self.tape_viewport.ascii_values ^= true,
                    'c' => self.on_exit(),
                    _ => (),
                },
                KeyCode::F(1) => self.on_set_input(),
                KeyCode::F(2) => self.on_set_auto_input(),
                KeyCode::Backspace
                | KeyCode::Delete
                | KeyCode::Enter
                | KeyCode::Tab
                | KeyCode::Char(_)
                    if !event.is_ctrl() && !event.is_alt() =>
                {
                    restart_interpreter = true
                }
                _ => (),
            }
        }

        restart_interpreter
    }

    fn on_exit(&mut self) {
        if false {
            self.dialogue = Some(Box::new(Dialogue::confirm_unsaved_changes(
                "Warning:\n\n\
                    There are unsaved changes, are you sure you want to quit?",
            )));
        } else {
            self.should_quit = true;
        }
    }

    fn on_save(&mut self) {
        match self.get_file_path() {
            None => self.on_save_as(),
            Some(path) => {
                let res = File::create(path).and_then(|mut file| {
                    for line in self.code.lines() {
                        file.write_all(line.as_bytes())?;
                        file.write_all(&[b'\n'])?;
                    }
                    Ok(())
                });
                if let Err(err) = res {
                    self.dialogue = Some(Box::new(Dialogue::error(format!(
                        "Error while saving file: {path}\n\n{err}",
                    ))));
                } else {
                    self.clean_hash = self.code.hash();
                }
            }
        }
    }

    fn on_save_as(&mut self) {
        self.dialogue = Some(Box::new(Dialogue::file_save_as(
            self.get_file_path().map(str::to_string),
        )));
    }

    fn on_set_input(&mut self) {
        self.dialogue = Some(Box::new(Dialogue::script_input()));
    }

    fn on_set_auto_input(&mut self) {
        self.dialogue = Some(Box::new(Dialogue::script_auto_input()));
    }
}
