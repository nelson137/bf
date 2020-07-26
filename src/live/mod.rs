use pancurses::{
    endwin, initscr, noecho, raw, start_color, Input::*, Window, COLOR_BLACK,
    COLOR_CYAN,
};
use structopt::StructOpt;

use crate::field::Field;
use crate::interpreter::{tape::Tape, Interpreter};
use crate::subcmd::SubCmd;
use crate::util::{die, is_valid_width, Style};

const ABOUT: &str = "Live scripting playground";
const WIDTH_HELP: &str = "The maximum width of the terminal for formatting \
                          the tape output.";
const ASCII_HELP: &str = "Show the ASCII characters in the tape output \
                          instead of the decimal values.";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct LiveCli {
    #[structopt(short, long, validator=is_valid_width, help=WIDTH_HELP)]
    width: Option<u32>,

    #[structopt(short, long, help=ASCII_HELP)]
    ascii_values: bool,
}

impl SubCmd for LiveCli {
    fn run(mut self) {
        let mut code = Field::new();
        let window = initscr();

        window.keypad(true);
        raw();
        noecho();
        start_color();
        Style::Cursor.init(COLOR_BLACK, COLOR_CYAN);
        Style::ControlHint.init(COLOR_BLACK, COLOR_CYAN);

        let width = window.get_max_x();

        // Print the initial tape state
        window.mv(1, 0);
        Tape::new()
            .chunks(width)
            .nc_display(&window, " ", self.ascii_values);
        draw_ui(&window, 1, 0);
        window.refresh();
        window.mv(5, 1);

        loop {
            let c = window.getch().unwrap_or_else(|| {
                die("failed to read from stdin".to_string())
            });

            match c {
                Character(c) => match c {
                    'q' | '\u{3}' => break,
                    '\u{1}' => self.ascii_values = !self.ascii_values,
                    '\u{8}' => code.cursor_home(), // home
                    '\u{c}' => code.cursor_end(),  // end
                    '>' | '<' | '+' | '-' | '[' | ']' | '.' | ',' => {
                        code.insert(c);
                    }
                    _ => (),
                },
                KeyLeft => code.cursor_left(),
                KeyRight => code.cursor_right(),
                KeyBackspace => code.backspace(),
                KeyDC => code.delete(),
                _ => (),
            }

            let mut interpreter = Interpreter::new(code.as_bytes(), "");
            while interpreter.next().is_some() {}

            let mut chunks = interpreter.tape.chunks(width);
            let n_chunks = chunks.len();
            let cursor_y = (2 + n_chunks * 3) as i32;
            let cursor_x = (1 + code.cursor()) as i32;

            // Print tape
            window.mv(1, 0);
            chunks.nc_display(&window, " ", self.ascii_values);

            // Print code
            window.mvprintw(cursor_y, 1, code.data());
            window.clrtoeol();

            draw_ui(&window, n_chunks, 5);

            window.refresh();
            window.mv(cursor_y, cursor_x);
        }

        endwin();
    }
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
    const CONTROLS: [[&str; 2]; 5] = [
        ["^S", "Save"],
        ["^C", "!Quit"],
        ["^A", "Toggle ASCII"],
        ["^H", "Begin"],
        ["^L", "End"],
    ];
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
    let divider_y = (1 + n_chunks * 3) as i32;
    window.mvprintw(divider_y, 0, "├");
    print_horizontal();
    window.printw("┤");

    // Divider 2
    let divider_y = height as i32 - 3 - output_lines as i32;
    window.mvprintw(divider_y, 0, "├");
    print_horizontal();
    window.printw("┤");
}
