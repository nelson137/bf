use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use atty::{self, Stream};
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
const SHOW_HELP: &str = "Show the tape during execution. Use -d/--delay to \
                         slow down execution.";
const WIDTH_HELP: &str = "The maximum width of the terminal for formatting \
                          the tape output.";
const ASCII_HELP: &str = "Show the ASCII characters in the tape output \
                          instead of the decimal values.";
const INFILE_HELP: &str = "The path to the Brainfuck script to execute. Can \
                           be a hyphen (-) to read the script from stdin.";
const OUTFILE_HELP: &str = "Print the final output of the program to outfile \
                            rather than dynamically showing it as it is \
                            printed. If outfile is a hyphen (-) or is omitted \
                            print to stdout.";

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

    #[structopt(short, long, help=SHOW_HELP)]
    show_tape: bool,

    #[structopt(short, long, validator=is_valid_width, help=WIDTH_HELP)]
    width: Option<u32>,

    #[structopt(short, long, help=ASCII_HELP)]
    ascii_values: bool,

    #[structopt(short, long, help=OUTFILE_HELP)]
    outfile: Option<Option<PathBuf>>,

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

    let mut final_output_only = !atty::is(Stream::Stdout);

    let outfile = if let Some(o_value) = args.outfile {
        final_output_only = true;
        o_value.filter(|p| *p != PathBuf::from("-"))
    } else {
        None
    };

    let mut printer = Printer::new();

    if args.show_tape {
        printer.print(interpreter.tape.display(width, args.ascii_values));
    }

    while interpreter.next().is_some() {
        printer.reset();
        if args.show_tape {
            sleep(Duration::from_millis(args.delay));
            printer.print(interpreter.tape.display(width, args.ascii_values));
        }
        if !final_output_only {
            printer.print(interpreter.output.clone());
        }
    }

    if final_output_only {
        let mut output_writer: Box<dyn Write> = match outfile {
            Some(path) => Box::new(
                File::create(path).unwrap_or_else(|err| die(err.to_string())),
            ),
            None => Box::new(io::stdout()),
        };
        output_writer
            .write_all(interpreter.output.as_bytes())
            .unwrap_or_else(|err| die(err.to_string()));
    }
}
