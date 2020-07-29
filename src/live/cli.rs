use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use pancurses::{
    endwin, has_colors, initscr, noecho, raw, resize_term, start_color,
    Input::*, Window, A_BOLD, COLOR_BLACK, COLOR_CYAN, COLOR_GREEN, COLOR_RED,
};
use structopt::StructOpt;

use crate::interpreter::Interpreter;
use crate::read::read_script;
use crate::subcmd::SubCmd;
use crate::ui::Style;
use crate::util::{die, is_valid_width};

use super::field::Field;

const ABOUT: &str = "Live scripting playground";
const WIDTH_HELP: &str = "The maximum width of the terminal for formatting \
                          the tape output";
const ASCII_HELP: &str = "Show the ASCII characters in the tape output \
                          instead of the decimal values";
const INFILE_HELP: &str = "The script to edit in live mode";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct LiveCli {
    #[structopt(short, long, validator=is_valid_width, help=WIDTH_HELP)]
    width: Option<u32>,

    #[structopt(short, long, help=ASCII_HELP)]
    ascii_values: bool,

    #[structopt(help=INFILE_HELP)]
    infile: Option<PathBuf>,
}

impl SubCmd for LiveCli {
    fn run(self) {
        Live::new(self.width, self.ascii_values, self.infile).run();
    }
}

struct Live {
    window: Window,
    width: Option<u32>,
    ascii_values: bool,
    infile: Option<PathBuf>,
    code: Field,
    frame_delay: Duration,
}

impl Live {
    fn new(
        width: Option<u32>,
        ascii_values: bool,
        infile: Option<PathBuf>,
    ) -> Self {
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
        }

        let code = if let Some(path) = &infile {
            let script = read_script(&path).unwrap_or_else(|e| die(e));
            Field::from(&String::from_utf8_lossy(&script))
        } else {
            Field::new()
        };

        Self {
            window,
            width,
            ascii_values,
            infile,
            code,
            frame_delay: Duration::from_millis(10),
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
                KeyEnter => self.code.enter(),

                // Characters and incorrect control keys
                Character(c) => {
                    match c {
                        // Enter
                        '\r' => self.code.enter(),
                        // ^C
                        '\u{3}' => break,
                        // ^A
                        '\u{1}' => self.ascii_values ^= true,
                        // Backspace
                        '\u{8}' => self.code.backspace(),
                        // Other
                        _ => self.code.insert(c),
                    }
                }

                // Arrow keys
                KeyLeft => self.code.cursor_left(),
                KeyRight => self.code.cursor_right(),
                KeyUp => self.code.cursor_up(),
                KeyDown => self.code.cursor_down(),

                // Control keys
                KeyBackspace => self.code.backspace(),
                KeyDC => self.code.delete(),
                KeyHome => self.code.cursor_home(),
                KeyEnd => self.code.cursor_end(),
                KeyPPage => self.code.cursor_top(),
                KeyNPage => self.code.cursor_bottom(),

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
        let mut interpreter =
            Interpreter::new(self.code.text().as_bytes(), "");
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
        self.window.mv(1, 1);
        let (color, msg) = match status {
            Ok(msg) => (Style::StatusOk.get(), msg),
            Err(msg) => (Style::StatusErr.get(), msg),
        };
        self.window.attron(color);
        self.window.attron(A_BOLD);
        self.window.printw("Status: ");
        self.window.printw(msg);
        self.window.attroff(A_BOLD);
        self.window.attroff(color);
        self.window.clrtoeol();

        // Print tape
        self.window.mv(2, 0);
        chunks.nc_display(&self.window, " ", self.ascii_values);

        let code_y = (3 + n_chunks * 3) as i32;
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

        self.draw_ui(n_chunks, output_lines);

        self.window.refresh();

        // Move window cursor to cursor position in code
        let code_cursor = self.code.cursor();
        let y = code_y + code_cursor.0 as i32;
        let x = code_x + code_cursor.1 as i32;
        self.window.mv(y, x);
    }

    fn draw_ui(&self, n_chunks: usize, output_lines: usize) {
        let (height, width) = self.window.get_max_yx();
        let print_horizontal =
            || (2..width).map(|_| self.window.printw("─")).last();

        // Top
        self.window.mvprintw(0, 0, "┌");
        print_horizontal();
        self.window.printw("┐");

        // Left and right
        for y in 1..height - 2 {
            self.window.mvprintw(y, 0, "│");
            self.window.mvprintw(y, width - 1, "│");
        }

        // Bottom
        self.window.printw("└");
        print_horizontal();
        self.window.printw("┘");

        // Controls
        const CONTROLS: [[&str; 2]; 2] =
            [["^C", "!Quit"], ["^A", "Toggle ASCII"]];
        CONTROLS.iter().for_each(|[map, hint]| {
            self.window.attron(Style::ControlHint.get());
            self.window.printw(map);
            self.window.attroff(Style::ControlHint.get());
            self.window.printw(":");
            self.window.printw(hint);
            self.window.printw("  ");
        });
        self.window.clrtoeol();

        // Divider 1
        let divider_y = (2 + n_chunks * 3) as i32;
        self.window.mvprintw(divider_y, 0, "├");
        print_horizontal();
        self.window.printw("┤");

        // Divider 2
        let divider_y = height as i32 - 3 - output_lines as i32;
        self.window.mvprintw(divider_y, 0, "├");
        print_horizontal();
        self.window.printw("┤");
    }
}
