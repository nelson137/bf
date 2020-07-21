use std::io::{self, Read};

use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use structopt::StructOpt;

use crate::interpreter::{tape::Tape, Interpreter};
use crate::printer::Printer;
use crate::subcmd::SubCmd;
use crate::util::{die, get_width, is_valid_width};

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

fn getchar() -> Result<char, String> {
    let c = io::stdin()
        .bytes()
        .next()
        .ok_or("end of stdin".to_string())?
        .map_err(|err| err.to_string())?;
    Ok(c as char)
}

impl SubCmd for LiveCli {
    fn run(self) {
        const CTRL_C: char = 3 as char;
        const BACKSPACE: char = 8 as char;

        let width = get_width(self.width);

        let mut code = String::new();
        let mut printer = Printer::new();

        enable_raw_mode().unwrap_or_else(|err| die(err.to_string()));

        // Print the initial tape state
        printer.print(Tape::new().display(width, self.ascii_values));

        loop {
            let c = getchar().unwrap_or_else(|err| die(err));

            match c {
                CTRL_C | 'q' => break,
                '>' | '<' | '+' | '-' | '[' | ']' | '.' | ',' => code.push(c),
                BACKSPACE => {
                    code.pop();
                }
                _ => (),
            }

            printer.reset();

            let mut interpreter =
                Interpreter::new(code.bytes().collect(), String::new());
            while interpreter.next().is_some() {}

            printer.print(interpreter.tape.display(width, self.ascii_values));
            printer.print(code.clone());
        }

        disable_raw_mode().unwrap_or_else(|err| die(err.to_string()));
    }
}
