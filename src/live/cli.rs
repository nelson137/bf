use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use pancurses::{
    chtype, endwin, has_colors, initscr, noecho, raw, resize_term,
    start_color, Input::*, Window, A_BOLD, COLOR_BLACK, COLOR_CYAN,
    COLOR_GREEN, COLOR_RED,
};
use structopt::StructOpt;

use crate::interpreter::Interpreter;
use crate::read::read_script;
use crate::subcmd::SubCmd;
use crate::ui::Style;
use crate::util::{die, EOL};

use super::editable::{Field, TextArea};

const ABOUT: &str = "Live scripting playground";
const ASCII_HELP: &str = "Show the ASCII characters in the tape output \
                          instead of the decimal values";
const INFILE_HELP: &str = "The script to edit in live mode";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct LiveCli {
    #[structopt(short, long, help=ASCII_HELP)]
    ascii_values: bool,

    #[structopt(help=INFILE_HELP)]
    infile: Option<PathBuf>,
}

impl SubCmd for LiveCli {
    fn run(self) {
        Live::new(self.ascii_values, self.infile).run();
    }
}

struct Live {
    window: Window,
    ascii_values: bool,
    infile: Option<PathBuf>,
    original_script: String,
    code: TextArea,
    frame_delay: Duration,
}

const WARN_UNSAVED_CHANGES: &str =
    "Warning: there are unsaved changes, are you sure you want to exit \
    [y/N]? ";
const ERROR_EMPTY_FILENAME: &str = "Error: filename cannot be empty";

impl Live {
    fn new(ascii_values: bool, infile: Option<PathBuf>) -> Self {
        let window = initscr();
        window.keypad(true);
        window.nodelay(true);
        raw();
        noecho();

        if has_colors() {
            start_color();
            Style::Cursor.init(COLOR_BLACK, COLOR_CYAN);
            Style::ControlHint.init(COLOR_BLACK, COLOR_CYAN);
            Style::StatusOk.init(COLOR_GREEN, COLOR_BLACK);
            Style::StatusErr.init(COLOR_RED, COLOR_BLACK);
            Style::Info.init(COLOR_GREEN, COLOR_BLACK);
        }

        let (code, original_script) = if let Some(path) = &infile {
            let script_raw = read_script(&path).unwrap_or_else(|e| die(e));
            let script = String::from_utf8_lossy(&script_raw).into_owned();
            (TextArea::from(&script), script)
        } else {
            (TextArea::new(), String::from(EOL))
        };

        Self {
            window,
            ascii_values,
            infile,
            original_script,
            code,
            frame_delay: Duration::from_millis(10),
        }
    }

    fn is_dirty(&self) -> bool {
        self.code.text() != self.original_script
    }

    fn can_exit_safely(&self) -> bool {
        !self.is_dirty() || self.prompt_yn(WARN_UNSAVED_CHANGES)
    }

    fn style_do<F: FnOnce() -> i32>(&self, attr: chtype, f: F) {
        if has_colors() {
            self.window.attron(attr);
        }
        f();
        if has_colors() {
            self.window.attroff(attr);
        }
    }

    fn run(&mut self) {
        self.draw();

        loop {
            sleep(self.frame_delay);

            let input = match self.window.getch() {
                Some(i) => i,
                None => continue,
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
                KeyBackspace | Character('\u{8}') => self.code.backspace(),
                KeyDC => self.code.delete(),

                // Insertions and commands
                KeyEnter | Character('\r') => self.code.enter(),
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
                    resize_term(0, 0);
                    self.window.clear();
                }

                // Other
                _ => (),
            }

            self.draw();
        }

        endwin();
    }

    fn draw(&self) {
        let (height, width) = self.window.get_max_yx();
        const STATUS_OK: &str = "ok";
        let mut status = Ok(STATUS_OK.to_string());

        // Run the script
        let mut interpreter = Interpreter::new(self.code.text(), "");
        while let Some(frame) = interpreter.next() {
            if let Err(err) = frame {
                status = Err(err);
                break;
            }
        }

        // One character is lost on either side from the border
        let mut chunks = interpreter.tape.chunks(width - 2);
        let n_chunks = chunks.len();

        let output: String = interpreter
            .output
            .chars()
            .filter(|c| c.is_ascii_whitespace() || c.is_ascii_graphic())
            .collect();
        let output_lines = output.lines().count();

        // Print the file name
        if let Some(path) = &self.infile {
            self.style_do(A_BOLD, || {
                self.window.mvprintw(0, 0, path.display().to_string())
            });
        }
        self.window.clrtoeol();

        // Print status
        self.window.mv(2, 1);
        let (color, msg) = match status {
            Ok(msg) => (Style::StatusOk.get(), msg),
            Err(msg) => (Style::StatusErr.get(), msg),
        };
        self.style_do(color + A_BOLD, || {
            self.window.printw("Status: ");
            self.window.printw(msg)
        });
        self.window.clrtoeol();

        // Print tape
        self.window.mv(3, 0);
        chunks.nc_display(&self.window, " ", self.ascii_values);

        let code_y = (4 + n_chunks * 3) as i32;
        let code_x = 1;

        // Print code
        self.window.mv(code_y, 0);
        for line in self.code.lines() {
            self.window.printw(" ");
            self.window.printw(line);
            self.window.printw("\n");
        }

        // Print output
        self.window.mv(height - output_lines as i32 - 2, 0);
        for line in output.lines() {
            self.window.printw(" ");
            self.window.printw(line);
            self.window.printw("\n");
        }

        self.draw_border(n_chunks, output_lines);

        // Controls
        const CONTROLS: [[&str; 2]; 4] = [
            ["^S", "Save"],
            ["^X", "Save As"],
            ["^C", "Quit"],
            ["^A", "Toggle ASCII"],
        ];
        self.window.mv(height - 1, 0);
        CONTROLS.iter().for_each(|[map, hint]| {
            self.style_do(Style::ControlHint.get(), || {
                self.window.printw(map)
            });
            self.window.printw(":");
            self.window.printw(hint);
            self.window.printw("  ");
        });
        self.window.clrtoeol();

        self.window.refresh();

        // Move window cursor to cursor position in code
        let code_cursor = self.code.cursor();
        let y = code_y + code_cursor.0 as i32;
        let x = code_x + code_cursor.1 as i32;
        self.window.mv(y, x);
    }

    fn draw_border(&self, n_chunks: usize, output_lines: usize) {
        let (height, width) = self.window.get_max_yx();
        let print_horizontal =
            || (2..width).map(|_| self.window.printw("─")).last();

        // Top
        self.window.mvprintw(1, 0, "┌");
        print_horizontal();
        self.window.printw("┐");

        // Left and right
        for y in 2..height - 2 {
            self.window.mvprintw(y, 0, "│");
            self.window.mvprintw(y, width - 1, "│");
        }

        // Bottom
        self.window.printw("└");
        print_horizontal();
        self.window.printw("┘");

        // Divider 1
        let divider_y = (3 + n_chunks * 3) as i32;
        self.window.mvprintw(divider_y, 0, "├");
        print_horizontal();
        self.window.printw("┤");

        // Divider 2
        let divider_y = height as i32 - 3 - output_lines as i32;
        self.window.mvprintw(divider_y, 0, "├");
        print_horizontal();
        self.window.printw("┤");
    }

    fn info_msg<S: AsRef<str>>(&self, msg: S) {
        let height = self.window.get_max_y();

        self.window.mvprintw(height - 1, 0, msg);
        self.style_do(Style::Info.get(), || {
            self.window.printw("  Press ENTER")
        });
        self.window.refresh();

        loop {
            sleep(self.frame_delay);

            if let Some(input) = self.window.getch() {
                match input {
                    KeyEnter | Character('\r') => break,
                    Character('\u{3}') => break,
                    _ => (),
                }
            }
        }
    }

    fn prompt_yn<S: AsRef<str>>(&self, msg: S) -> bool {
        let msg_len = msg.as_ref().len() as i32;
        let height = self.window.get_max_y();

        self.window.mvprintw(height - 1, 0, &msg);
        self.window.refresh();

        let mut response: Option<char> = None;

        loop {
            sleep(self.frame_delay);

            if let Some(input) = self.window.getch() {
                match input {
                    Character('\u{3}') => break,
                    KeyEnter | Character('\r') if response.is_some() => break,
                    KeyBackspace | Character('\u{8}') => response = None,
                    Character(c) => match c {
                        '\u{1b}' | '\u{3}' => break, // Esc | ^C
                        'y' | 'Y' | 'n' | 'N' => response = Some(c),
                        _ => (),
                    },
                    _ => (),
                }

                self.window.mvprintw(height - 1, 0, &msg);
                if let Some(c) = response {
                    self.window.mvprintw(height - 1, msg_len, c.to_string());
                }
                self.window.clrtoeol();
            }
        }

        match response {
            Some('y') | Some('Y') => true,
            _ => false,
        }
    }

    fn save(&mut self) {
        let path = if let Some(p) = &self.infile {
            p
        } else {
            self.save_as();
            return;
        };

        let window_bak = self.window.dupwin();

        let result = match File::create(path) {
            Ok(mut file) => file.write_all(self.code.text().as_bytes()),
            Err(err) => Err(err),
        };

        if let Err(err) = result {
            self.info_msg(format!("Error: failed to save file: {}", err));
        }

        self.window.overwrite(&window_bak);
        self.window.refresh();
    }

    fn save_as(&mut self) {
        let height = self.window.get_max_y();
        let window_bak = self.window.dupwin();
        let mut field = Field::new();

        let draw = |path| {
            self.window.mvprintw(height - 1, 0, "Filename: ");
            self.window.printw(path);
            self.window.clrtoeol();
            self.window.refresh();
        };

        draw(String::new());

        loop {
            sleep(self.frame_delay);

            let input = match self.window.getch() {
                Some(i) => i,
                None => continue,
            };

            match input {
                KeyEnter | Character('\r') => {
                    let path = field.text().trim();
                    if path.is_empty() {
                        self.info_msg(ERROR_EMPTY_FILENAME);
                    } else {
                        self.infile = Some(PathBuf::from(path));
                    }
                    break;
                }
                KeyBackspace | Character('\u{8}') => field.backspace(),
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
            self.window.mv(height - 1, 10 + field.cursor() as i32);
        }

        self.window.overlay(&window_bak);
        if self.infile.is_some() {
            self.save();
        }
    }
}
