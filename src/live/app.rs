use std::{
    fs::File,
    io::{stdout, Write},
    iter,
    path::PathBuf,
    thread,
    time::Duration,
};

use crossterm::{
    event::*,
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
    err::BfResult,
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
    ascii_values: bool,
    file_path: Option<PathBuf>,
    should_quit: SharedBool,
    spinner: Spinner,
    code: TextArea,
    input: String,
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
    pub fn new(cli: LiveCli) -> BfResult<Self> {
        enable_raw_mode()?;
        execute!(stdout(), EnableMouseCapture, EnterAlternateScreen)?;

        let file_contents = if let Some(path) = &cli.infile {
            String::from_utf8_lossy(&read_script_file(&path)?).into_owned()
        } else {
            String::new()
        };

        Ok(Self {
            ascii_values: cli.ascii_values,
            file_path: cli.infile,
            should_quit: SharedBool::new(false),
            spinner: Spinner::default(),
            code: TextArea::from(&file_contents),
            input: String::new(),
            clean_hash: sha1_digest(&file_contents),
            event_queue: EventQueue::new().with_tick_delay(100),
            delay: Duration::from_millis(20),
            dialogue: None,
            async_interpreter: AsyncInterpreter::new(
                file_contents.clone(),
                String::new(),
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

    pub fn run(&mut self) -> BfResult<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let mut restart_interpreter: bool;

        while !self.should_quit.load() {
            restart_interpreter = false;

            terminal.draw(|f| self.draw(f))?;

            for event in self.event_queue.pop_all() {
                let event = match event {
                    BfEvent::Tick => {
                        self.spinner.tick();
                        continue;
                    }
                    BfEvent::Input(Event::Key(e)) => e,
                    _ => continue,
                };

                if let Some(dialogue) = &mut self.dialogue {
                    match dialogue.on_event(event) {
                        Decision::Waiting => (),
                        Decision::No => self.dialogue = None,
                        Decision::Yes => {
                            dialogue.run_action();
                            self.dialogue = None;
                        }
                        Decision::Input(input) => {
                            match dialogue.get_reason() {
                                Reason::Filename => {
                                    self.file_path =
                                        Some(PathBuf::from(input));
                                    self.dialogue = None;
                                    self.on_save();
                                }
                                Reason::Input => {
                                    self.input = input.into();
                                    self.dialogue = None;
                                    restart_interpreter = true;
                                }
                                _ => (),
                            }
                        }
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
            }

            if restart_interpreter {
                let status = self.async_interpreter.state().status;
                if let Status::FatalError(fe) = status {
                    let mut dialogue = ButtonDialogue::error(fe);
                    dialogue.set_reason(Reason::Info);
                    self.dialogue = Some(Box::new(dialogue));
                }
                self.async_interpreter
                    .restart(self.code.text(), self.input.clone())?;
            }

            thread::yield_now();
            thread::sleep(self.delay);
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
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

    fn draw_content(&self, frame: &mut Frame, area: Rect, int_state: State) {
        let output = String::from_utf8_lossy(&int_state.output).into_owned();
        let output_lines = output.split_terminator('\n').count() as u16;

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(4),            // Tape
                Constraint::Length(1),            // Divider
                Constraint::Min(2),               // Editor
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

        let num_width = self.code.lines().count().count_digits().max(3) as u16;
        let line_width = content_area.width.saturating_sub(1 + num_width);
        let wrapped_lines = self.code.wrapped_num_lines(line_width as usize);

        let num_str = |i| format!("{:>w$}", i, w = num_width as usize);
        let num_style = Style::default().fg(Color::Yellow);
        let rows: Vec<Row> = wrapped_lines
            .iter()
            .flat_map(|(i, line_chunks)| {
                iter::once(Span::styled(num_str(i), num_style))
                    .chain(iter::repeat(Span::raw("")))
                    .zip(line_chunks)
                    .map(|(num, chunk)| Row::new(vec![num, Span::raw(chunk)]))
            })
            .collect();
        let widths = [Constraint::Length(num_width), Constraint::Min(0)];
        let table = Table::new(rows).widths(&widths);
        frame.render_widget(table, content_area);

        let cursor = self.code.cursor();
        let cur_x = cursor.1 % line_width as usize;
        let cur_y = wrapped_lines
            .iter()
            .take(cursor.0)
            .map(|(_, line_chunks)| line_chunks.len())
            .sum::<usize>()
            + (cursor.1 / line_width as usize);
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
        const CONTROLS: [[&str; 2]; 5] = [
            ["^S", "Save"],
            ["^X", "Save As"],
            ["^C", "Quit"],
            ["^A", "Toggle ASCII"],
            ["F1", "Set Input"],
        ];
        let keys_style = Style::default().fg(Color::Black).bg(Color::Cyan);

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
}
