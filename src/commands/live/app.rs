use std::{
    borrow::Cow,
    fs::File,
    io::{stderr, stdout, Write},
    path::{Path, PathBuf},
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
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};
use ratatui_textarea::TextArea;

use crate::{
    util::{
        common::{sha1_digest, Sha1Digest},
        read::read_script_file,
        sync::SharedBool,
        tui::{sublayouts, BfEvent, EventQueue, Frame, KeyEventExt, Terminal},
    },
    widgets::Spinner,
};

use super::{
    async_interpreter::{AsyncInterpreter, Status},
    cli::LiveCli,
    dialogue::{
        centered_rect, ButtonDialogue, Decision, Dialogue, PromptStrDialogue,
        Reason,
    },
    textarea::TextAreaExts,
    widgets::{
        Footer, Header, TapeViewport, TapeViewportState, VerticalStack,
    },
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

pub struct App<'textarea> {
    term_width: usize,
    term_height: usize,
    file_path: Option<PathBuf>,
    should_quit: SharedBool,
    spinner: Spinner,
    code: TextArea<'textarea>,
    tape_viewport_state: TapeViewportState,
    input: String,
    auto_input: Option<u8>,
    clean_hash: Sha1Digest,
    event_queue: EventQueue,
    delay: Duration,
    dialogue: Option<Box<dyn Dialogue>>,
    async_interpreter: AsyncInterpreter,
}

impl Drop for App<'_> {
    fn drop(&mut self) {
        reset_terminal();
    }
}

impl App<'_> {
    pub fn new(cli: LiveCli) -> Result<Self> {
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
            file_path: cli.infile,
            should_quit: SharedBool::new(false),
            spinner: Spinner::default(),
            code,
            tape_viewport_state: TapeViewportState::new(cli.ascii_values),
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

    fn get_file_path(&self) -> Option<&Path> {
        self.file_path.as_deref()
    }

    fn get_file_path_string(&self) -> Option<Cow<str>> {
        self.file_path.as_deref().map(Path::to_string_lossy)
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

        while !self.should_quit.load() {
            restart_interpreter = false;

            terminal.draw(|f| self.draw(f))?;

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
                    let mut dialogue = ButtonDialogue::error(fe);
                    dialogue.set_reason(Reason::Info);
                    self.dialogue = Some(Box::new(dialogue));
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

    fn handle_key_event(&mut self, event: KeyEvent) -> bool {
        let mut restart_interpreter = false;

        if let Some(dialogue) = &mut self.dialogue {
            match dialogue.on_event(event) {
                Decision::Waiting => (),
                Decision::No => self.dialogue = None,
                Decision::Yes => {
                    dialogue.run_action();
                    self.dialogue = None;
                }
                Decision::Input(input) => match dialogue.get_reason() {
                    Reason::Filename => {
                        self.file_path = Some(PathBuf::from(input));
                        self.dialogue = None;
                        self.on_save();
                    }
                    Reason::Input => {
                        self.input = input;
                        self.dialogue = None;
                        restart_interpreter = true;
                    }
                    Reason::AutoInput => {
                        self.auto_input = input.as_bytes().first().copied();
                        self.dialogue = None;
                        restart_interpreter = true;
                    }
                    _ => (),
                },
            }
        } else {
            self.code.on_event_multi_line(event);
            match event.code {
                KeyCode::Char(c) if event.is_ctrl() => match c {
                    's' => self.on_save(),
                    'x' => self.on_save_as(),
                    'a' => self.tape_viewport_state.ascii_values ^= true,
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

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.size();
        let state = self.async_interpreter.state();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Min(7),
                Constraint::Length(1),
            ])
            .split(area);
        sublayouts!([header_area, content_area, footer_area] = layout);

        self.draw_header(frame, header_area, state.status.clone());
        self.draw_content(frame, content_area, &state.output);
        self.draw_footer(frame, footer_area);

        if let Some(dialogue) = &self.dialogue {
            let dialogue_area = centered_rect(50, 50, area);
            dialogue.draw(frame, dialogue_area);
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect, status: Status) {
        Header::default()
            .is_dirty(self.is_dirty())
            .file_path(self.get_file_path_string())
            .status(status)
            .spinner(self.spinner)
            .render_(frame, area);
    }

    fn draw_content(&mut self, frame: &mut Frame, area: Rect, output: &[u8]) {
        let output = String::from_utf8_lossy(output);
        let output_lines = output.split_terminator('\n').count() as u16;

        let output_title = if output.ends_with('\n') {
            " Output "
        } else {
            " Output (no EOL) "
        };

        let stack = VerticalStack::<3>::new(
            [
                Constraint::Length(3),            // Tape
                Constraint::Min(1),               // Editor
                Constraint::Length(output_lines), // Output
            ],
            [" Tape ", " Code ", output_title],
            area,
        );

        let [tape_area, editor_area, output_area] = stack.areas();

        frame.render_widget(stack, area);

        // Tape
        let interpreter_state = self.async_interpreter.state();
        let widget = TapeViewport::new(&interpreter_state.tape);
        frame.render_stateful_widget(
            widget,
            tape_area,
            &mut self.tape_viewport_state,
        );

        // Editor
        frame.render_widget(self.code.widget(), editor_area);

        // Output
        if !output.is_empty() {
            let p = Paragraph::new(output);
            frame.render_widget(p, output_area);
        }
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Footer, area);
    }

    fn on_exit(&mut self) {
        if false {
            let should_quit = self.should_quit.clone();
            let mut dialogue = ButtonDialogue::confirm(
                "Warning:\n\nThere are unsaved changes, are you sure you want \
                    to quit?",
            );
            dialogue.set_reason(Reason::Confirm);
            dialogue.set_action(Box::new(move || should_quit.store(true)));
            self.dialogue = Some(Box::new(dialogue));
        } else {
            self.should_quit.store(true);
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
                    let mut dialogue = ButtonDialogue::error(format!(
                        "Error while saving file: {}\n\n{}",
                        path.display(),
                        err
                    ));
                    dialogue.set_reason(Reason::Info);
                    self.dialogue = Some(Box::new(dialogue));
                } else {
                    self.clean_hash = self.code.hash();
                }
            }
        }
    }

    fn on_save_as(&mut self) {
        let mut dialogue = PromptStrDialogue::new(
            " Save As ",
            "Filename: ",
            self.get_file_path_string().as_deref(),
        );
        dialogue.set_reason(Reason::Filename);
        self.dialogue = Some(Box::new(dialogue));
    }

    fn on_set_input(&mut self) {
        let mut dialogue = PromptStrDialogue::new(" Input ", "Input: ", None);
        dialogue.set_reason(Reason::Input);
        self.dialogue = Some(Box::new(dialogue));
    }

    fn on_set_auto_input(&mut self) {
        let mut dialogue = PromptStrDialogue::new(
            " Auto-Input ",
            "Input (only the first byte will be used): ",
            None,
        );
        dialogue.set_reason(Reason::AutoInput);
        self.dialogue = Some(Box::new(dialogue));
    }
}
