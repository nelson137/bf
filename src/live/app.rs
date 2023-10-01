use std::{
    fs::File,
    io::{stderr, stdout, Write},
    path::PathBuf,
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
    style::{Color, Modifier, Style, Styled, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};
use ratatui_textarea::TextArea;

use crate::util::{
    common::{sha1_digest, Sha1Digest},
    read::read_script_file,
    sync::SharedBool,
    tui::{
        sublayouts, BfEvent, EventQueue, Frame, KeyEventExt, Spinner, Terminal,
    },
};

use super::{
    async_interpreter::{AsyncInterpreter, State, Status},
    cli::LiveCli,
    dialogue::{
        centered_rect, ButtonDialogue, Decision, Dialogue, PromptStrDialogue,
        Reason,
    },
    textarea::TextAreaExts,
    widgets::VerticalStack,
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
    ascii_values: bool,
    file_path: Option<PathBuf>,
    should_quit: SharedBool,
    spinner: Spinner,
    code: TextArea<'textarea>,
    tape_viewport_start: usize,
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

        let interpreter_code = code.to_string();

        Ok(Self {
            term_width: 0,
            term_height: 0,
            ascii_values: cli.ascii_values,
            file_path: cli.infile,
            should_quit: SharedBool::new(false),
            spinner: Spinner::default(),
            code,
            tape_viewport_start: 0,
            input: String::new(),
            auto_input: None,
            clean_hash: sha1_digest(script_raw),
            event_queue: EventQueue::with_ticks(100),
            delay: Duration::from_millis(20),
            dialogue: None,
            async_interpreter: AsyncInterpreter::new(
                interpreter_code,
                String::new(),
                None,
            ),
        })
    }

    fn get_file_path(&self) -> Option<String> {
        self.file_path
            .as_ref()
            .map(|path| path.display().to_string())
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
                    self.code.to_string(),
                    self.input.clone(),
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
                    'a' => self.ascii_values ^= true,
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

        self.draw_header(frame, header_area, state.clone());
        self.draw_content(frame, content_area, state);
        self.draw_footer(frame, footer_area);

        if let Some(dialogue) = &self.dialogue {
            let dialogue_area = centered_rect(50, 50, area);
            dialogue.draw(frame, dialogue_area);
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect, int_state: State) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(1),  // Dirty indicator
                Constraint::Length(1),  // Spacer (skip)
                Constraint::Min(0),     // Filename
                Constraint::Length(1),  // Spacer (skip)
                Constraint::Length(18), // Status (max status length)
                Constraint::Length(1),  // Spacer (skip)
                Constraint::Length(1),  // Spinner
            ])
            .split(area);
        sublayouts!(
            [indicator_area, _, fn_area, _, status_area, _, spinner_area] =
                layout
        );

        // Draw dirty indicator
        if self.is_dirty() {
            frame.render_widget(Paragraph::new("*"), indicator_area);
        }

        // Draw filename
        let p = Paragraph::new(match self.get_file_path() {
            Some(path) => Span::raw(path),
            None => "New File".add_modifier(Modifier::ITALIC),
        });
        frame.render_widget(p, fn_area);

        // Draw status
        let status = int_state.status;
        let style = Style::default().add_modifier(Modifier::BOLD);
        let style = match status {
            Status::Done => Style::default(),
            Status::Running => style.fg(Color::Green),
            Status::WaitingForInput => style.fg(Color::Yellow),
            Status::Error(_) => style.fg(Color::Red),
            Status::FatalError(_) => style.fg(Color::Red),
        };
        frame.render_widget(
            Paragraph::new(status.to_string().set_style(style)),
            status_area,
        );

        // Draw spinner
        if status == Status::Running {
            frame.render_widget(self.spinner, spinner_area);
        }
    }

    fn draw_content(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        int_state: State,
    ) {
        let output = String::from_utf8_lossy(&int_state.output).into_owned();
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
        self.draw_content_tape(frame, tape_area);
        self.draw_content_editor(frame, editor_area);
        self.draw_content_output(frame, output_area, output);
    }

    fn draw_content_tape(&mut self, frame: &mut Frame, area: Rect) {
        let tape = self.async_interpreter.state().tape;
        let max_cells = (area.width as f32 / 4.0).ceil() as usize;
        let cursor_min = self.tape_viewport_start + 3;
        let cursor_max =
            self.tape_viewport_start + max_cells.saturating_sub(3);

        if tape.cursor() < cursor_min && self.tape_viewport_start > 0 {
            self.tape_viewport_start = self
                .tape_viewport_start
                .saturating_sub(cursor_min - tape.cursor());
        } else if tape.cursor() > cursor_max {
            self.tape_viewport_start += tape.cursor() - cursor_max;
        }

        let window = tape.window(
            self.tape_viewport_start,
            max_cells,
            self.ascii_values,
        );
        frame.render_widget(window, area);
    }

    fn draw_content_editor(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(self.code.widget(), area);
    }

    fn draw_content_output(
        &self,
        frame: &mut Frame,
        area: Rect,
        output: String,
    ) {
        if !output.is_empty() {
            let p = Paragraph::new(output);
            frame.render_widget(p, area);
        }
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        const CONTROLS: [[&str; 2]; 6] = [
            ["^S", "Save"],
            ["^X", "Save As"],
            ["^C", "Quit"],
            ["^A", "Toggle ASCII"],
            ["F1", "Set Input"],
            ["F2", "Set Auto-Input"],
        ];
        let keys_style = Style::default().bg(Color::Cyan).fg(Color::Black);

        let text = Line::from(
            CONTROLS
                .iter()
                .flat_map(|[keys, desc]| {
                    vec![
                        keys.set_style(keys_style),
                        Span::from(":"),
                        Span::from(*desc),
                        Span::from("  "),
                    ]
                })
                .collect::<Vec<_>>(),
        );
        frame.render_widget(Paragraph::new(text), area);
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
                let res = File::create(&path).and_then(|mut file| {
                    for line in self.code.lines() {
                        file.write_all(line.as_bytes())?;
                        file.write_all(&[b'\n'])?;
                    }
                    Ok(())
                });
                if let Err(err) = res {
                    let mut dialogue = ButtonDialogue::error(format!(
                        "Error while saving file: {}\n\n{}",
                        &path, err
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
            self.get_file_path().as_deref(),
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
