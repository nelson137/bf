use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use structopt::StructOpt;

mod util;
use util::die;

mod read;
use read::read_script;

mod interpreter;
use interpreter::Interpreter;

const DELAY_HELP: &str = "The delay, in milliseconds, between the evaluation \
                          of each Brainfuck instruction. Does nothing if \
                          --show-tape is not given.";
const INPUT_HELP: &str = "The input to provide the Brainfuck program for the \
                          read (,) instruction.";
const DUMP_HELP: &str = "Print the final state of the tape after execution.";
const SHOW_HELP: &str = "Show the tape during execution. Use -d/--delay to \
                         slow down execution.";
const ASCII_ONLY_HELP: &str = "Only use ASCII characters for output.";
const WIDTH_HELP: &str = "The maximum width of the terminal for formatting \
                          the tape output.";
const INFILE_HELP: &str = "The path to the Brainfuck script to execute. Can \
                           be a hyphen (-) to read the script from stdin.";

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

fn is_valid_width(value: String) -> Result<(), String> {
    match value.parse::<i64>() {
        Ok(n) => {
            if n < 5 {
                Err("value must be an integer > 5".to_string())
            } else {
                Ok(())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Debug, StructOpt)]
struct Cli {
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

    #[structopt(short="t", long, conflicts_with="show-tape", help=DUMP_HELP)]
    dump_tape: bool,

    #[structopt(short, long, help=SHOW_HELP)]
    show_tape: bool,

    #[structopt(short, long, help=ASCII_ONLY_HELP)]
    ascii_only: bool,

    #[structopt(short, long, validator=is_valid_width, help=WIDTH_HELP)]
    width: Option<u32>,

    #[structopt(parse(from_os_str), help=INFILE_HELP)]
    infile: PathBuf,
}

fn main() {
    let args = Cli::from_args();

    let script = read_script(&args.infile).unwrap_or_else(|e| die(e));

    let width: u32 = match args.width {
        Some(w) => w,
        None => match term_size::dimensions() {
            Some((w, _h)) if w > 5 => w as u32,
            _ => 65, // Wide enough for 16 cells
        },
    };

    let mut interpreter = Interpreter::new(script, args.input).unwrap_or_else(|err| die(err));

    if args.show_tape {
        interpreter.tape.print(width, args.ascii_only);
    }

    while interpreter.next().is_some() {
        if args.show_tape {
            thread::sleep(Duration::from_millis(args.delay));
            interpreter.tape.print(width, args.ascii_only);
        }
    }

    if args.dump_tape {
        interpreter.tape.print(width, args.ascii_only);
    }
}
