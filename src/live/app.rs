use std::{
    fs::File,
    io::{Write, stdout},
    path::PathBuf,
    thread,
    time::Duration,
};

use crossterm::{
    event::*,
    execute,
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
        disable_raw_mode,
        enable_raw_mode,
    },
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::{
    read::read_script_file,
    tui_util::*,
    util::{BfResult, Sha1Digest, StrExt, sha1_digest},
};

use super::{
    cli::LiveCli,
    dialogue::*,
    editable::{Editable, TextArea},
};

#[derive(PartialEq)]
enum DialogueFor {
    Null,
    DirtyExit,
    SaveAs,
    Info,
}

pub struct App {
    ascii_values: bool,
    file_path: Option<PathBuf>,
    code: TextArea,
    clean_hash: Sha1Digest,
    event_queue: EventQueue,
    delay: Duration,
    dialogue_for: DialogueFor,
    dialogue: Option<Box<dyn Dialogue>>,
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
            code: TextArea::from(&file_contents),
            clean_hash: sha1_digest(&file_contents),
            event_queue: EventQueue::new(),
            delay: Duration::from_millis(5),
            dialogue: None,
            dialogue_for: DialogueFor::Null,
        })
    }

    fn get_file_path(&self) -> Option<String> {
        self.file_path.as_ref().map(|path| path.display().to_string())
    }

    fn is_dirty(&self) -> bool {
        self.code.hash() != self.clean_hash
    }

    pub fn run(&mut self) -> BfResult<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        'main: loop {
            terminal.draw(|f| self.draw(f))?;

            while let Some(event) = self.event_queue.pop() {
                let event = if let BfEvent::Input(Event::Key(e)) = event {
                    e
                } else {
                    continue;
                };

                if let Some(dialogue) = &mut self.dialogue {
                    match dialogue.on_event(event) {
                        DialogueDecision::Waiting => (),
                        DialogueDecision::No => {
                            self.dialogue_for = DialogueFor::Null;
                            self.dialogue = None;
                        }
                        DialogueDecision::Yes => {
                            if self.dialogue_for == DialogueFor::DirtyExit {
                                break 'main;
                            }
                            self.dialogue_for = DialogueFor::Null;
                            self.dialogue = None;
                        }
                        DialogueDecision::Input(input) => {
                            self.file_path = Some(PathBuf::from(input));
                            self.on_save();
                        }
                    }
                } else {
                    self.code.on_event(event);
                    match event.code {
                        KeyCode::Char(c) if event.is_ctrl() => match c {
                            's' => self.on_save(),
                            'x' => self.on_save_as(),
                            'a' => self.ascii_values ^= true,
                            'c' => {
                                if self.is_dirty() {
                                    self.dialogue_for = DialogueFor::DirtyExit;
                                    self.dialogue = Some(Box::new(
                                        ButtonDialogue::confirm(
                                            "Warning: there are unsaved \
                                            changes, are you sure you want to \
                                            exit?"
                                        )
                                    ));
                                } else {
                                    break 'main;
                                }
                            }
                            _ => (),
                        }
                        _ => (),
                    }
                }
            }

            thread::sleep(self.delay);
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.size();

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(1),
                Constraint::Min(7),
                Constraint::Length(1),
            ])
            .split(area);
        self.draw_header(frame, sections[0]);
        self.draw_content(frame, sections[1]);
        self.draw_footer(frame, sections[2]);

        if let Some(dialogue) = &self.dialogue {
            let dialogue_area = centered_rect(50, 50, area);
            dialogue.draw(frame, dialogue_area);
        }
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        // Draw filename
        if let Some(path) = self.get_file_path() {
            let status = if self.is_dirty() {
                format!("* {}", path)
            } else {
                path
            };
            let p = Paragraph::new(Span::from(status));
            frame.render_widget(p, area);
        }
    }

    fn draw_content(&self, frame: &mut Frame, area: Rect) {
        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(4),
                Constraint::Length(1),
                Constraint::Min(2),
            ])
            .split(area);
        match sections.as_slice() {
            [tape_area, divider_area, editor_area] => {
                self.draw_content_tape(frame, *tape_area);
                self.draw_content_divider(frame, *divider_area);
                self.draw_content_editor(frame, *editor_area);
            }
            _ => panic!("failed to split content area into tape and editor"),
        }
    }

    fn draw_content_tape(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT);
        let content_area = block.inner(area);
        frame.render_widget(block, area);

        let content = Block::default()
            .style(Style::default().bg(Color::DarkGray));
        frame.render_widget(content, content_area);
    }

    fn draw_content_divider(&self, frame: &mut Frame, area: Rect) {
        let inner_width = area.width as usize - 2;
        let symbols = BorderType::line_symbols(BorderType::Plain);
        let divider =
            symbols.vertical_right.to_owned()
            + &symbols.horizontal.repeated(inner_width)
            + symbols.vertical_left;
        frame.render_widget(Paragraph::new(divider), area);
    }

    fn draw_content_editor(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM);
        let content_area = block.inner(area);
        frame.render_widget(block, area);

        let lines = self.code
            .lines()
            .map(|line| Spans::from(line.as_ref()))
            .collect::<Vec<_>>();

        let p = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(p, content_area);

        let (cur_x, cur_y) = self.code.cursor();
        frame.set_cursor(
            content_area.x + cur_y as u16,
            content_area.y + cur_x as u16
        );
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        const CONTROLS: [[&str; 2]; 4] = [
            ["^S", "Save"],
            ["^X", "Save As"],
            ["^C", "Quit"],
            ["^A", "Toggle ASCII"],
        ];
        let desc_style = Style::default().fg(Color::Black).bg(Color::Cyan);

        let text = Spans::from(
            CONTROLS.iter().map(|[keys, desc]| vec![
                Span::styled(*keys, desc_style),
                Span::from(":"),
                Span::from(*desc),
                Span::from("  "),
            ].into_iter()).flatten().collect::<Vec<_>>()
        );
        let p = Paragraph::new(text).block(Block::default());
        frame.render_widget(p, area);
    }

    fn on_save(&mut self) {
        match self.get_file_path() {
            None => self.on_save_as(),
            Some(path) => {
                let res = File::create(&path)
                    .and_then(|mut file| {
                        file.write_all(self.code.text().as_bytes())
                    });
                if let Err(err) = res {
                    self.dialogue_for = DialogueFor::Info;
                    self.dialogue = Some(Box::new(ButtonDialogue::error(
                        format!(
                            "Error while saving file: {}\n\n{}",
                            &path,
                            err
                        )
                    )));
                } else {
                    self.clean_hash = self.code.hash();
                }
            }
        }
    }

    fn on_save_as(&mut self) {
        self.dialogue_for = DialogueFor::SaveAs;
        self.dialogue = Some(Box::new(PromptStrDialogue::new(
            " Save As ",
            "Filename: ",
            self.get_file_path().as_deref()
        )));
    }

}
