use std::{
    collections::VecDeque,
    ffi::OsStr,
    fmt::Write as FmtWrite,
    fs::File,
    io::{stderr, stdout, Write as IoWrite},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use anyhow::Result;
use bf_tui::{
    async_interpreter::{AsyncInterpreter, Status},
    events::{BfEvent, EventQueue, KeyEventExt},
    widgets::{
        live::{
            AppWidget, Dialogue, DialogueCommand, TapeViewportState,
            TextAreaExts,
        },
        Spinner,
    },
    Terminal,
};
use bf_utils::hash::{sha1_digest, Sha1Digest};
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
use tracing::{trace, warn};
use tui_textarea::TextArea;

use crate::utils::read::read_script_file;

use super::{cli::LiveCli, logging::init_logging};

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
    file_path_abs: Option<String>,
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
            .flat_map(String::as_bytes)
            .copied()
            .collect::<Vec<_>>();

        let mut code = TextArea::from(script);
        code.set_line_number_style(Style::default().fg(Color::Yellow));
        code.set_cursor_line_style(Style::default());

        let interpreter_code = code.bytes().collect();

        let mut this = Self {
            term_width: 0,
            term_height: 0,
            file_path: None,
            file_path_abs: None,
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
                VecDeque::default(),
                None,
            ),
        };

        this.set_file_path(cli.infile.map(|p| p.to_string_lossy().into()));

        Ok(this)
    }

    fn get_file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    fn get_file_path_abs(&self) -> Option<&str> {
        self.file_path_abs.as_deref()
    }

    fn set_file_path(&mut self, maybe_path: Option<String>) {
        if let Some(path_str) = maybe_path.as_deref() {
            let path = Path::new(&path_str);
            self.file_path_abs = Some(if path.is_absolute() {
                path_str.to_string()
            } else {
                match path.canonicalize() {
                    Ok(canon) => canon.to_string_lossy().to_string(),
                    Err(err) => {
                        warn!(path = %path_str, "Unable to canonicalize path, using relative: {err}");
                        path_str.to_string()
                    }
                }
            });
            self.file_path = maybe_path;
        } else {
            self.file_path_abs = None;
            self.file_path = None;
        }
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
                            restart_interpreter = self.handle_key_event(e);
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
            editor: &self.code,
            dialogue: self.dialogue.as_deref(),
            file_path: self.get_file_path(),
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
                    self.set_file_path(Some(path));
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
                    restart_interpreter = true;
                }
                _ => (),
            }
        }

        restart_interpreter
    }

    fn on_exit(&mut self) {
        if self.is_dirty() {
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
        let initial_path = self.get_file_path_abs().map(|path_abs_str| {
            let mut path_abs = PathBuf::from(path_abs_str);
            let mut stem = path_abs
                .file_stem()
                .expect("File path has no file name")
                .to_string_lossy()
                .into_owned();
            stem.reserve(9); // Pre-allocate for the appended "-copy-N"
            let ext = path_abs.extension().map(OsStr::to_os_string);

            stem.push_str("-copy");
            path_abs.set_file_name(&stem);
            if let Some(ext) = &ext {
                path_abs.set_extension(ext);
            }

            if !path_abs.exists() {
                return path_abs.to_string_lossy().to_string();
            }

            stem.push('-');
            let stem_len_without_n = stem.len();

            let mut n = 2_u32;

            loop {
                stem.truncate(stem_len_without_n);
                if let Err(err) = write!(&mut stem, "{n}") {
                    warn!(n, string = ?stem, "Failed to write number to string: {err}");
                    return path_abs_str.to_string();
                }

                path_abs.set_file_name(&stem);
                if let Some(ext) = &ext {
                    path_abs.set_extension(ext);
                }

                if !path_abs.exists() {
                    break;
                }

                n += 1;
            }

            path_abs.to_string_lossy().to_string()
        });

        self.dialogue = Some(Box::new(Dialogue::file_save_as(initial_path)));
    }

    fn on_set_input(&mut self) {
        self.dialogue = Some(Box::new(Dialogue::script_input()));
    }

    fn on_set_auto_input(&mut self) {
        self.dialogue = Some(Box::new(Dialogue::script_auto_input()));
    }
}
