use std::{
    fs::File,
    io::{stdout, Write},
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
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Row, Table},
};

use crate::util::{
    common::{sha1_digest, Sha1Digest, USizeExt},
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
    editable::{Editable, TextArea},
};

pub struct App {
    term_height: usize,
    ascii_values: bool,
    file_path: Option<PathBuf>,
    should_quit: SharedBool,
    spinner: Spinner,
    code: TextArea,
    input: String,
    auto_input: Option<u8>,
    clean_hash: Sha1Digest,
    event_queue: EventQueue,
    delay: Duration,
    dialogue: Option<Box<dyn Dialogue>>,
    async_interpreter: AsyncInterpreter,
}

impl Drop for App {
    fn drop(&mut self) {
        execute!(stdout(), DisableMouseCapture, LeaveAlternateScreen).ok();
        disable_raw_mode().ok();
    }
}

impl App {
    pub fn new(cli: LiveCli) -> Result<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnableMouseCapture, EnterAlternateScreen)?;

        let file_contents = if let Some(path) = &cli.infile {
            String::from_utf8_lossy(&read_script_file(&path)?).into_owned()
        } else {
            String::new()
        };

        Ok(Self {
            term_height: 0,
            ascii_values: cli.ascii_values,
            file_path: cli.infile,
            should_quit: SharedBool::new(false),
            spinner: Spinner::default(),
            code: TextArea::from(&file_contents, 0),
            input: String::new(),
            auto_input: None,
            clean_hash: sha1_digest(&file_contents),
            event_queue: EventQueue::with_ticks(100),
            delay: Duration::from_millis(20),
            dialogue: None,
            async_interpreter: AsyncInterpreter::new(
                file_contents.clone(),
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

        self.term_height = terminal.size()?.height as usize;

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
                        Event::Resize(_, height) => {
                            self.term_height = height as usize
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
                    self.code.text(),
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
                        self.input = input.into();
                        self.dialogue = None;
                        restart_interpreter = true;
                    }
                    Reason::AutoInput => {
                        self.auto_input = input.as_bytes().first().map(|b| *b);
                        self.dialogue = None;
                        restart_interpreter = true;
                    }
                    _ => (),
                },
            }
        } else {
            self.code.on_event(event);
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
            None => Span::styled(
                "New File",
                Style::default().add_modifier(Modifier::ITALIC),
            ),
        });
        frame.render_widget(p, fn_area);

        // Draw status
        let status = int_state.status;
        let style = Style::default().add_modifier(Modifier::BOLD);
        let style = match status {
            Status::Done => style,
            Status::Running => style.fg(Color::Green),
            Status::WaitingForInput => style.fg(Color::Yellow),
            Status::Error(_) => style.fg(Color::Red),
            Status::FatalError(_) => style.fg(Color::Red),
        };
        frame.render_widget(
            Paragraph::new(Span::styled(status.to_string(), style)),
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

        self.code.resize_viewport(
            self.term_height.saturating_sub(output_lines as usize + 9),
        );

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(4),            // Tape
                Constraint::Length(1),            // Divider
                Constraint::Min(1),               // Editor
                Constraint::Length(1),            // Divider
                Constraint::Length(output_lines), // Output
                Constraint::Length(1),            // Bottom
            ])
            .split(area);

        sublayouts!(
            [
                tape_area,
                divider_area1,
                editor_area,
                divider_area2,
                output_area,
                bottom_area
            ] = layout
        );

        let output_title = if output.ends_with('\n') {
            " Output "
        } else {
            " Output (no EOL) "
        };

        self.draw_content_tape(frame, tape_area);
        self.draw_content_divider(frame, divider_area1, " Code ");
        self.draw_content_editor(frame, editor_area);
        self.draw_content_divider(frame, divider_area2, output_title);
        self.draw_content_output(frame, output_area, output);
        self.draw_content_bottom(frame, bottom_area);
    }

    fn draw_content_tape(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title(" Tape ");
        let tape_area = block.inner(area);
        frame.render_widget(block, area);

        let tape = self.async_interpreter.state().tape;
        let max_cells = (tape_area.width as f32 / 4f32).ceil() as usize;
        let window = tape.window(0, max_cells, self.ascii_values);
        frame.render_widget(window, tape_area);
    }

    fn draw_content_divider(
        &self,
        frame: &mut Frame,
        area: Rect,
        title: &str,
    ) {
        let inner_width =
            (area.width as usize).saturating_sub(2 + title.len());
        let symbols = BorderType::line_symbols(BorderType::Plain);
        let divider = symbols.vertical_right.to_owned()
            + title
            + &symbols.horizontal.repeat(inner_width)
            + symbols.vertical_left;
        frame.render_widget(Paragraph::new(divider), area);
    }

    fn draw_content_editor(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::LEFT | Borders::RIGHT);
        let content_area = block.inner(area);
        frame.render_widget(block, area);

        let num_width =
            self.code.viewport().len().count_digits().max(3) as u16;
        let line_width = content_area.width.saturating_sub(1 + num_width);
        let editor_lines =
            self.code.wrapped_numbered_lines(line_width as usize);

        let fmt_num = |n| format!("{:>1$}", n, num_width as usize);
        let num_style = Style::default().fg(Color::Yellow);
        let rows = editor_lines.map(|(maybe_n, line_chunk)| {
            let n_span = match maybe_n {
                Some(n) => Span::styled(fmt_num(n), num_style),
                _ => Span::raw(""),
            };
            Row::new(vec![n_span, Span::raw(line_chunk)])
        });
        let widths = [Constraint::Length(num_width), Constraint::Min(0)];
        let table = Table::new(rows).widths(&widths);
        frame.render_widget(table, content_area);

        let cursor = self.code.cursor();
        let cur_x = cursor.1 % line_width as usize;
        let cur_y = self.code.cursor().0 + (cursor.1 / line_width as usize);
        frame.set_cursor(
            content_area.x + num_width + 1 + cur_x as u16,
            content_area.y + cur_y as u16,
        );
    }

    fn draw_content_output(
        &self,
        frame: &mut Frame,
        area: Rect,
        output: String,
    ) {
        if output.len() > 0 {
            let block =
                Block::default().borders(Borders::LEFT | Borders::RIGHT);
            let p = Paragraph::new(output).block(block);
            frame.render_widget(p, area);
        }
    }

    fn draw_content_bottom(&self, frame: &mut Frame, area: Rect) {
        let inner_width = (area.width as usize).saturating_sub(2);
        let symbols = BorderType::line_symbols(BorderType::Plain);
        let bottom = symbols.bottom_left.to_owned()
            + &symbols.horizontal.repeat(inner_width)
            + symbols.bottom_right;
        frame.render_widget(Paragraph::new(bottom), area);
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

        let text = Spans::from(
            CONTROLS
                .iter()
                .flat_map(|[keys, desc]| {
                    vec![
                        Span::styled(*keys, keys_style),
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
        if self.is_dirty() {
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
                    file.write_all(self.code.text().as_bytes())
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
