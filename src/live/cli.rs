use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use pancurses::{
    endwin, has_colors, initscr, noecho, raw, resize_term, start_color,
    Input::*, Window, A_BOLD, COLOR_BLACK, COLOR_CYAN, COLOR_GREEN, COLOR_RED,
    COLOR_YELLOW,
};
use structopt::StructOpt;

use crate::interpreter::Interpreter;
use crate::read::read_script;
use crate::subcmd::SubCmd;
use crate::ui::{style_do, Style};
use crate::util::{die, is_valid_infile};

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

    #[structopt(validator=is_valid_infile, help=INFILE_HELP)]
    infile: Option<PathBuf>,
}

impl SubCmd for LiveCli {
    fn run(self) {
        Live::new(self.ascii_values, self.infile).run();
    }
}

struct Live {
    win: Window,
    win_header: Window,
    win_content: Window,
    win_footer: Window,
    ascii_values: bool,
    file_path: Option<PathBuf>,
    code: TextArea,
    frame_delay: Duration,
}

const ERROR_CREATE_WINDOW: &str = "Error: failed to create windows";
const ERROR_EMPTY_FILENAME: &str = "Error: filename cannot be empty";

impl Live {
    fn new(ascii_values: bool, file_path: Option<PathBuf>) -> Self {
        let code = if let Some(path) = &file_path {
            let script_raw = read_script(&path).unwrap_or_else(|e| die(e));
            TextArea::from(String::from_utf8_lossy(&script_raw))
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

        Self {
            win,
            win_header,
            win_content,
            win_footer,
            ascii_values,
            file_path,
            code,
            frame_delay: Duration::from_millis(10),
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
        self.draw_content();

        loop {
            sleep(self.frame_delay);

            let input = match self.win_content.getch() {
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
                    self.win.clear();
                }

                // Other
                _ => (),
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

    fn draw_content(&self) {
        let (height, width) = self.win_content.get_max_yx();
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

        // Print status
        self.win_content.mv(1, 1);
        let (color, msg) = match status {
            Ok(msg) => (Style::StatusOk.get(), msg),
            Err(msg) => (Style::StatusErr.get(), msg),
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

            if let Some(input) = self.win_footer.getch() {
                match input {
                    KeyEnter | Character('\r') => break,
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

            if let Some(input) = self.win_footer.getch() {
                match input {
                    KeyEnter | Character('\r') => {
                        if response.is_some() {
                            break;
                        }
                    }
                    KeyBackspace | Character('\u{8}') => response = None,
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

            let input = match self.win_footer.getch() {
                Some(i) => i,
                None => continue,
            };

            match input {
                KeyEnter | Character('\r') => {
                    let path = field.text().trim();
                    if path.is_empty() {
                        self.info_msg(ERROR_EMPTY_FILENAME);
                    } else {
                        self.file_path = Some(PathBuf::from(path));
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
            self.win_footer.mv(0, 10 + field.cursor() as i32);
        }

        self.draw_header();
        self.draw_footer();
        if self.file_path.is_some() {
            self.save();
        }
    }
}
