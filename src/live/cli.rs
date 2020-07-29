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
    fn run(mut self) {
        let mut code = if let Some(path) = &self.infile {
            let script = read_script(&path).unwrap_or_else(|e| die(e));
            Field::from(&String::from_utf8_lossy(&script))
        } else {
            Field::new()
        };

        let window = initscr();
        let frame_delay = Duration::from_millis(10);

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

        draw(&window, &code, self.ascii_values);

        loop {
            sleep(frame_delay);

            let input = match window.getch() {
                Some(i) => i,
                None => continue,
            };

            match input {
                KeyEnter => code.enter(),

                // Characters and incorrect control keys
                Character(c) => {
                    match c {
                        // Enter
                        '\r' => code.enter(),
                        // ^C
                        '\u{3}' => break,
                        // ^A
                        '\u{1}' => self.ascii_values ^= true,
                        // Backspace
                        '\u{8}' => code.backspace(),
                        // Other
                        _ => code.insert(c),
                    }
                }

                // Arrow keys
                KeyLeft => code.cursor_left(),
                KeyRight => code.cursor_right(),
                KeyUp => code.cursor_up(),
                KeyDown => code.cursor_down(),

                // Control keys
                KeyBackspace => code.backspace(),
                KeyDC => code.delete(),
                KeyHome => code.cursor_home(),
                KeyEnd => code.cursor_end(),
                KeyPPage => code.cursor_top(),
                KeyNPage => code.cursor_bottom(),

                // Terminal resize
                KeyResize => {
                    resize_term(0, 0);
                    window.clear();
                }

                // Other
                _ => (),
            }

            draw(&window, &code, self.ascii_values);
        }

        endwin();
    }
}

fn sanitize_output(data: &str) -> String {
    data.chars()
        .filter(|c| c.is_ascii_whitespace() || c.is_ascii_graphic())
        .collect()
}

fn draw(window: &Window, code: &Field, ascii_values: bool) {
    let (height, width) = window.get_max_yx();

    const STATUS_OK: &str = "ok";
    let mut status = Ok(STATUS_OK.to_string());

    // Run the script
    let mut interpreter = Interpreter::new(code.text().as_bytes(), "");
    while let Some(frame) = interpreter.next() {
        if let Err(err) = frame {
            status = Err(err);
            break;
        }
    }

    // One character is lost on either side from the border
    let mut chunks = interpreter.tape.chunks(width - 2);
    let n_chunks = chunks.len();

    let code_cursor = code.cursor();

    let output = sanitize_output(&interpreter.output);
    let output_lines = output.lines().count();

    // Print status
    window.mv(1, 1);
    let (color, msg) = match status {
        Ok(msg) => (Style::StatusOk.get(), msg),
        Err(msg) => (Style::StatusErr.get(), msg),
    };
    window.attron(color);
    window.attron(A_BOLD);
    window.printw("Status: ");
    window.printw(msg);
    window.attroff(A_BOLD);
    window.attroff(color);
    window.clrtoeol();

    // Print tape
    window.mv(2, 0);
    chunks.nc_display(&window, " ", ascii_values);

    let code_y = (3 + n_chunks * 3) as i32;
    let code_x = 1;

    // Print code
    window.mv(code_y, 0);
    for line in code.lines() {
        window.printw(" ");
        window.printw(line);
        window.printw("\n");
    }

    // Print output
    window.mv(height - output_lines as i32 - 2, 0);
    for line in output.lines() {
        window.printw(" ");
        window.printw(line);
        window.printw("\n");
    }

    draw_ui(&window, n_chunks, output_lines);

    window.refresh();
    window.mv(code_y + code_cursor.0 as i32, code_x + code_cursor.1 as i32);
}

fn draw_ui(window: &Window, n_chunks: usize, output_lines: usize) {
    let (height, width) = window.get_max_yx();
    let print_horizontal = || (2..width).map(|_| window.printw("─")).last();

    // Top
    window.mvprintw(0, 0, "┌");
    print_horizontal();
    window.printw("┐");

    // Left and right
    for y in 1..height - 2 {
        window.mvprintw(y, 0, "│");
        window.mvprintw(y, width - 1, "│");
    }

    // Bottom
    window.printw("└");
    print_horizontal();
    window.printw("┘");

    // Controls
    const CONTROLS: [[&str; 2]; 2] = [["^C", "!Quit"], ["^A", "Toggle ASCII"]];
    CONTROLS.iter().for_each(|[map, hint]| {
        window.attron(Style::ControlHint.get());
        window.printw(map);
        window.attroff(Style::ControlHint.get());
        window.printw(":");
        window.printw(hint);
        window.printw("  ");
    });
    window.clrtoeol();

    // Divider 1
    let divider_y = (2 + n_chunks * 3) as i32;
    window.mvprintw(divider_y, 0, "├");
    print_horizontal();
    window.printw("┤");

    // Divider 2
    let divider_y = height as i32 - 3 - output_lines as i32;
    window.mvprintw(divider_y, 0, "├");
    print_horizontal();
    window.printw("┤");
}
