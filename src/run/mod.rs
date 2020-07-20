use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use structopt::StructOpt;

use crate::interpreter::Interpreter;
use crate::print::Printer;
use crate::subcmd::SubCmd;
use crate::util::{die, get_width, is_valid_width};

mod read;
use read::read_script;

const ABOUT: &str = "Execute a Brainfuck script";
const DELAY_HELP: &str = "The delay, in milliseconds, between the evaluation \
                          of each Brainfuck instruction. Does nothing if \
                          -s/--show-tape is not given.";
const INPUT_HELP: &str = "The input to provide the Brainfuck program for the \
                          read (,) instruction.";
const SHOW_HELP: &str = "Show the tape during execution. Use -d/--delay to \
                         slow down execution.";
const WIDTH_HELP: &str = "The maximum width of the terminal for formatting \
                          the tape output.";
const ASCII_HELP: &str = "Show the ASCII characters in the tape output \
                          instead of the decimal values.";
const INFILE_HELP: &str = "The path to the Brainfuck script to execute. Can \
                           be a hyphen (-) to read the script from stdin.";
const OUTFILE_HELP: &str = "The name of the file to which the final output \
                            of the Brainfuck script will be printed.";

fn is_valid_delay(value: String) -> Result<(), String> {
    match value.parse::<i64>() {
        Ok(n) => {
            if n < 0 {
                Err("value must be an integer >= 0".to_string())
            } else {
                Ok(())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct RunCli {
    #[structopt(
        short,
        long,
        default_value="0",
        validator=is_valid_delay,
        hide_default_value=true,
        help=DELAY_HELP
    )]
    delay: u64,

    #[structopt(
        short,
        long,
        default_value="",
        hide_default_value=true,
        help=INPUT_HELP
    )]
    input: String,

    #[structopt(short, long, help=SHOW_HELP)]
    show_tape: bool,

    #[structopt(short, long, validator=is_valid_width, help=WIDTH_HELP)]
    width: Option<u32>,

    #[structopt(short, long, help=ASCII_HELP)]
    ascii_values: bool,

    #[structopt(short, long, help=OUTFILE_HELP)]
    outfile: Option<PathBuf>,

    #[structopt(parse(from_os_str), help=INFILE_HELP)]
    infile: PathBuf,
}

impl SubCmd for RunCli {
    fn run(self) {
        let script = read_script(&self.infile).unwrap_or_else(|e| die(e));

        let width = get_width(self.width);

        let mut interpreter = Interpreter::new(script, self.input);

        let mut printer = Printer::new();

        if self.show_tape {
            printer.print(interpreter.tape.display(width, self.ascii_values));
        }

        while interpreter.next().is_some() {
            printer.reset();
            if self.show_tape {
                sleep(Duration::from_millis(self.delay));
                printer
                    .print(interpreter.tape.display(width, self.ascii_values));
            }
            printer.print(interpreter.output.clone());
        }

        if let Some(path) = self.outfile {
            File::create(path)
                .unwrap_or_else(|err| die(err.to_string()))
                .write_all(interpreter.output.as_bytes())
                .unwrap_or_else(|err| die(err.to_string()));
        }
    }
}
