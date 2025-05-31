use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{Sender, Receiver, channel};
use std::thread::{self, JoinHandle, sleep};
use std::time::{Duration, SystemTime};

use pancurses::{
    endwin, has_colors, initscr, noecho, raw, resize_term, start_color,
    Input::*, Window, A_BOLD, COLOR_BLACK, COLOR_CYAN, COLOR_GREEN, COLOR_RED,
    COLOR_YELLOW,
};

use crate::interpreter::{Interpreter, Tape};
use crate::read::read_script;
use crate::ui::{style_do, Style};
use crate::util::{die, normalize_input};
use crate::dprintln;

mod cli;
pub use cli::LiveCli;

mod editable;
use editable::{Field, TextArea};

struct Live {
    win: Window,
    win_header: Window,
    win_content: Window,
    win_footer: Window,
    ascii_values: bool,
    file_path: Option<PathBuf>,
    code: TextArea,
    frame_delay: Duration,
    interpreter_thread: Option<JoinHandle<()>>,
    stop_interpreter_flag: Arc<AtomicBool>,
    interpreter_sender: Sender<ContentState>,
    interpreter_receiver: Receiver<ContentState>,
}

struct ContentState {
    status: Status,
    tape: Tape,
    output: String,
}

enum Status {
    Running,
    Done,
    Error(String),
}

const ERROR_CREATE_WINDOW: &str = "Error: failed to create windows";
const ERROR_EMPTY_FILENAME: &str = "Error: filename cannot be empty";

impl Live {
    fn new(ascii_values: bool, file_path: Option<PathBuf>) -> Self {
        let code = if file_path.is_some() {
            let script = read_script(&file_path).unwrap_or_else(|e| die(e));
            TextArea::from(String::from_utf8_lossy(&script))
        } else {
            TextArea::new()
        };

        let win = initscr();
        win.keypad(true);
        win.nodelay(true);
        raw();
        noecho();

        let sub = |parent: &Window, nlines, ncols, begy, begx| {
            parent
                .subwin(nlines, ncols, begy, begx)
                .unwrap_or_else(|_| die(ERROR_CREATE_WINDOW.to_string()))
        };

        let (height, width) = win.get_max_yx();

        let win_header = sub(&win, 1, width, 0, 0);
        let win_content = sub(&win, height - 2, width, 1, 0);
        let win_footer = sub(&win, 1, width, height - 1, 0);

        if has_colors() {
            start_color();
            Style::Cursor.init(COLOR_BLACK, COLOR_CYAN);
            Style::ControlHint.init(COLOR_BLACK, COLOR_CYAN);
            Style::StatusOk.init(COLOR_GREEN, COLOR_BLACK);
            Style::StatusErr.init(COLOR_RED, COLOR_BLACK);
            Style::Info.init(COLOR_GREEN, COLOR_BLACK);
            Style::Warning.init(COLOR_YELLOW, COLOR_BLACK);
        }

        let (sender, receiver) = channel();

        Self {
            win,
            win_header,
            win_content,
            win_footer,
            ascii_values,
            file_path,
            code,
            interpreter_sender: sender,
            interpreter_receiver: receiver,
            interpreter_thread: None,
            frame_delay: Duration::from_millis(100),
            stop_interpreter_flag: Arc::new(AtomicBool::new(false)),
        }
    }

    fn can_exit_safely(&self) -> bool {
        if !self.code.is_dirty() {
            return true;
        }

        let msg_prefix = "Warning: ";
        let msg = "there are unsaved changes, are you sure you want to \
                   exit [y/N]? ";

        style_do(&self.win_footer, Style::Warning.get(), || {
            self.win_footer.mvprintw(0, 0, &msg_prefix)
        });
        self.win_footer.printw(msg);
        let ret = self.prompt_yn((msg_prefix.len() + msg.len()) as i32);

        self.draw_footer();
        ret
    }

    fn run(&mut self) {
        self.draw_header();
        self.draw_footer();
        self.interpret();
        self.draw_content();

        let mut code_changed: bool;

        loop {
            sleep(self.frame_delay);
            code_changed = false;

            if let Some(input) = normalize_input(self.win.getch()) {
                code_changed = match input {
                    KeyBackspace | KeyDC |
                        Character('+') | Character('-') |
                        Character('>') | Character('<') |
                        Character('[') | Character(']') |
                        Character('.') | Character(',') => true,
                    _ => false,
                };

                match input {
                    // Cursor movement
                    KeyLeft => self.code.cursor_left(),
                    KeyRight => self.code.cursor_right(),
                    KeyUp => self.code.cursor_up(),
                    KeyDown => self.code.cursor_down(),
                    KeyHome => self.code.cursor_home(),
                    KeyEnd => self.code.cursor_end(),
                    KeyPPage => self.code.cursor_top(),
                    KeyNPage => self.code.cursor_bottom(),

                    // Deletions
                    KeyBackspace => self.code.backspace(),
                    KeyDC => self.code.delete(),

                    // Insertions and commands
                    KeyEnter => self.code.enter(),
                    Character(c) => match c {
                        // ^C
                        '\u{3}' => {
                            if self.can_exit_safely() {
                                break;
                            }
                        }
                        // ^A
                        '\u{1}' => self.ascii_values ^= true,
                        // ^S
                        '\u{13}' => self.save(),
                        // ^X
                        '\u{18}' => self.save_as(),
                        // Other
                        _ => self.code.insert(c),
                    },

                    // Terminal resize
                    KeyResize => {
                        dprintln!("term resize");
                        resize_term(0, 0);
                        self.win.clear();
                        self.win.refresh();
                    }

                    // Other
                    _ => (),
                }
            }

            if code_changed {
                self.interpret();
            }
            self.draw_content();
        }

        endwin();
    }

    fn draw_header(&self) {
        // Print the file name
        if let Some(path) = &self.file_path {
            style_do(&self.win_header, A_BOLD, || {
                self.win_header.mvprintw(0, 0, path.display().to_string())
            });
        }
        self.win_header.clrtoeol();
        self.win_header.refresh();
    }

    fn interpret(&mut self) {
        if let Some(t) = self.interpreter_thread.take() {
            self.stop_interpreter_flag.store(true, Ordering::Relaxed);
            t.join().ok();
        }

        let t_code = self.code.text();
        let t_flag = self.stop_interpreter_flag.clone();
        let t_sender = self.interpreter_sender.clone();

        self.interpreter_thread = Some(thread::spawn(move || {
            dprintln!("begin interpret [");
            let now = SystemTime::now();

            let mut i_status = Status::Done;
            let mut interpreter = Interpreter::new(t_code, "");

            while t_flag.load(Ordering::Relaxed) {
                if let Ok(d) = now.elapsed() {
                    if d.as_millis() >= 200 {
                        dprintln!("send running status");
                        t_sender.send(ContentState {
                            status: Status::Running,
                            tape: Tape::new(),
                            output: String::new(),
                        }).ok();
                    }
                }
                match interpreter.next() {
                    None => break,
                    Some(Err(err)) => {
                        i_status = Status::Error(err);
                        break;
                    }
                    _ => ()
                }
            }

            t_sender.send(ContentState {
                status: i_status,
                tape: interpreter.tape,
                output: interpreter.output,
            }).ok();

            dprintln!("] end interpret");
        }));
    }

    fn draw_content(&mut self) {
        let (height, width) = self.win_content.get_max_yx();

        let value = self.interpreter_receiver.try_recv();
        let (status, tape, output) = if let Ok(state) = value {
            (state.status, state.tape, state.output)
        } else {
            (Status::Done, Tape::new(), String::new())
        };

        // One character is lost on either side from the border
        let mut chunks = tape.chunks(width - 2);
        let n_chunks = chunks.len();

        let output: String = output
            .chars()
            .filter(|c| c.is_ascii_whitespace() || c.is_ascii_graphic())
            .collect();
        let output_lines = output.lines().count();

        // Print status
        self.win_content.mv(1, 1);
        let (color, msg) = match status {
            Status::Running =>
                (Style::StatusOk.get(), String::from("Running...")),
            Status::Done => (Style::StatusOk.get(), String::from("Ok")),
            Status::Error(msg) => (Style::StatusErr.get(), msg),
        };
        style_do(&self.win_content, color + A_BOLD, || {
            self.win_content.printw("Status: ");
            self.win_content.printw(msg)
        });
        self.win_content.clrtoeol();

        // Print tape
        self.win_content.mv(2, 0);
        chunks.nc_display(&self.win_content, " ", self.ascii_values);

        let code_y = (3 + n_chunks * 3) as i32;
        let code_x = 1;

        // Print code
        self.win_content.mv(code_y, 0);
        for line in self.code.lines() {
            self.win_content.printw(" ");
            self.win_content.printw(line);
            self.win_content.printw("\n");
        }

        // Print output
        self.win_content.mv(height - output_lines as i32 - 1, 0);
        for line in output.lines() {
            self.win_content.printw(" ");
            self.win_content.printw(line);
            self.win_content.printw("\n");
        }

        self.draw_content_border(n_chunks, output_lines);

        // Move window cursor to cursor position in code
        let code_cursor = self.code.cursor();
        let y = code_y + code_cursor.0 as i32;
        let x = code_x + code_cursor.1 as i32;
        self.win_content.mv(y, x);

        self.win.refresh();
        self.win_content.refresh();
    }

    fn draw_content_border(&self, n_chunks: usize, output_lines: usize) {
        let (height, width) = self.win_content.get_max_yx();
        let print_horizontal =
            || (2..width).map(|_| self.win_content.printw("─")).last();

        // Top
        self.win_content.mvprintw(0, 0, "┌");
        print_horizontal();
        self.win_content.printw("┐");

        // Left and right
        for y in 1..height - 1 {
            self.win_content.mvprintw(y, 0, "│");
            self.win_content.mvprintw(y, width - 1, "│");
        }

        // Bottom
        self.win_content.printw("└");
        print_horizontal();
        self.win_content.printw("┘");

        // Divider 1 (tape/editor)
        let divider_y = (2 + n_chunks * 3) as i32;
        self.win_content.mvprintw(divider_y, 0, "├");
        print_horizontal();
        self.win_content.printw("┤");

        // Divider 2 (editor/output)
        let divider_y = height as i32 - 2 - output_lines as i32;
        self.win_content.mvprintw(divider_y, 0, "├");
        print_horizontal();
        self.win_content.printw("┤");
    }

    fn draw_footer(&self) {
        const CONTROLS: [[&str; 2]; 4] = [
            ["^S", "Save"],
            ["^X", "Save As"],
            ["^C", "Quit"],
            ["^A", "Toggle ASCII"],
        ];

        self.win_footer.mv(0, 0);

        CONTROLS.iter().for_each(|[map, hint]| {
            style_do(&self.win_footer, Style::ControlHint.get(), || {
                self.win_footer.printw(map)
            });
            self.win_footer.printw(":");
            self.win_footer.printw(hint);
            self.win_footer.printw("  ");
        });

        self.win_footer.clrtoeol();
        self.win_footer.refresh();
    }

    fn info_msg<S: AsRef<str>>(&self, msg: S) {
        self.win_footer.mvprintw(0, 0, msg);
        style_do(&self.win_footer, Style::Info.get(), || {
            self.win_footer.printw("  Press ENTER")
        });
        self.win_footer.refresh();

        loop {
            sleep(self.frame_delay);

            if let Some(input) = normalize_input(self.win_footer.getch()) {
                match input {
                    KeyEnter => break,
                    Character('\u{3}') => break,
                    _ => (),
                }
            }
        }
    }

    fn prompt_yn(&self, start_x: i32) -> bool {
        self.win_footer.refresh();

        let mut response: Option<char> = None;

        loop {
            sleep(self.frame_delay);

            if let Some(input) = normalize_input(self.win_footer.getch()) {
                match input {
                    KeyEnter => {
                        if response.is_some() {
                            break;
                        }
                    }
                    KeyBackspace => response = None,
                    Character(c) => match c {
                        '\u{1b}' | '\u{3}' => break, // Esc | ^C
                        'y' | 'Y' | 'n' | 'N' => response = Some(c),
                        _ => (),
                    },
                    _ => (),
                }

                self.win_footer.mv(0, start_x);
                if let Some(c) = response {
                    style_do(&self.win_footer, A_BOLD, || {
                        self.win_footer.printw(c.to_string())
                    });
                }
                self.win_footer.clrtoeol();
            }
        }

        match response {
            Some('y') | Some('Y') => true,
            _ => false,
        }
    }

    fn save(&mut self) {
        let path = if let Some(p) = &self.file_path {
            p
        } else {
            self.save_as();
            return;
        };

        let result = match File::create(path) {
            Ok(mut file) => file.write_all(self.code.text().as_bytes()),
            Err(err) => Err(err),
        };

        if let Err(err) = result {
            self.info_msg(format!("Error: failed to save file: {}", err));
        }

        self.code.save();
        self.draw_footer();
    }

    fn save_as(&mut self) {
        let mut field = Field::new();

        let draw = |path| {
            self.win_footer.mvprintw(0, 0, "Filename: ");
            self.win_footer.printw(path);
            self.win_footer.clrtoeol();
            self.win_footer.refresh();
        };

        draw(String::new());

        loop {
            sleep(self.frame_delay);

            let input = match normalize_input(self.win_footer.getch()) {
                Some(i) => i,
                None => continue,
            };

            match input {
                KeyEnter => {
                    let path = field.text().trim();
                    if path.is_empty() {
                        self.info_msg(ERROR_EMPTY_FILENAME);
                    } else {
                        self.file_path = Some(PathBuf::from(path));
                    }
                    break;
                }
                KeyBackspace => field.backspace(),
                KeyLeft => field.cursor_left(),
                KeyRight => field.cursor_right(),
                KeyHome => field.cursor_home(),
                KeyEnd => field.cursor_end(),
                KeyDC => field.delete(),
                Character(c) => match c {
                    '\u{1b}' | '\u{3}' => break, // Esc | ^C
                    _ => field.insert(c),
                },
                _ => (),
            }

            draw(String::from(field.text()));
            self.win_footer.mv(0, 10 + field.cursor() as i32);
        }

        self.draw_header();
        self.draw_footer();
        if self.file_path.is_some() {
            self.save();
        }
    }
}
