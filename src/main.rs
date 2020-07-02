use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use structopt::StructOpt;

mod interpreter;
use interpreter::Interpreter;

mod print;
use print::Printer;

mod read;
use read::read_script;

mod util;
use util::die;

const DELAY_HELP: &str = "The delay, in milliseconds, between the evaluation \
                          of each Brainfuck instruction. Does nothing if \
                          -s/--show-tape is not given.";
const INPUT_HELP: &str = "The input to provide the Brainfuck program for the \
                          read (,) instruction.";
const DUMP_HELP: &str = "Print the final state of the tape after execution.";
const SHOW_HELP: &str = "Show the tape during execution. Use -d/--delay to \
                         slow down execution.";
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

    #[structopt(short, long, validator=is_valid_width, help=WIDTH_HELP)]
    width: Option<u32>,

    #[structopt(parse(from_os_str), help=INFILE_HELP)]
    infile: PathBuf,
}

fn main() {
    #[cfg(windows)]
    match ansi_term::enable_ansi_support() {
        Ok(_) => (),
        Err(_code) => eprintln!("Warning: ANSI support not enabled"),
    }

    let args = Cli::from_args();

    let script = read_script(&args.infile).unwrap_or_else(|e| die(e));

    let width: u32 = match args.width {
        Some(w) => w,
        None => match term_size::dimensions() {
            Some((w, _h)) if w > 5 => w as u32,
            _ => 65, // Wide enough for 16 cells
        },
    };

    let mut interpreter =
        Interpreter::new(script, args.input).unwrap_or_else(|err| die(err));
    let mut printer = Printer::new();

    if args.show_tape {
        printer.print(interpreter.tape.draw(width));
    }

    while interpreter.next().is_some() {
        printer.reset();
        if args.show_tape {
            sleep(Duration::from_millis(args.delay));
            printer.print(interpreter.tape.draw(width));
        }
        printer.print(interpreter.output.clone());
    }

    if args.dump_tape {
        printer.reset();
        printer.print(interpreter.tape.draw(width));
        printer.print(interpreter.output.clone());
    }
}
