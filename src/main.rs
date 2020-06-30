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
                          of each Brainfuck instruction.";
const INPUT_HELP: &str = "The input to provide the Brainfuck program for the \
                          read (,) instruction.";
const DUMP_HELP: &str = "Print the final state of the tape after execution.";
const SHOW_HELP: &str = "Show the tape during execution. Use -d,--delay to \
                         slow down execution.";
const ASCII_ONLY_HELP: &str = "Only use ASCII characters for output.";
const WIDTH_HELP: &str = "The maximum width of the terminal for formatting \
                          the tape output.";
const INFILE_HELP: &str = "The path to the Brainfuck script to execute. Can \
                           be a hyphen (-) to read the script from stdin.";

fn is_pos_int(value: String) -> Result<(), String> {
    match value.parse::<i64>() {
        Ok(i) => {
            if i > 0 {
                Ok(())
            } else {
                Err("value must be an integer > 0".to_string())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short, long, default_value="0", validator=is_pos_int, help=DELAY_HELP)]
    delay: u64,

    #[structopt(short, long, default_value="", help=INPUT_HELP)]
    input: String,

    #[structopt(long, conflicts_with="show-tape", help=DUMP_HELP)]
    dump_tape: bool,

    #[structopt(long, help=SHOW_HELP)]
    show_tape: bool,

    #[structopt(short, long, help=ASCII_ONLY_HELP)]
    ascii_only: bool,

    #[structopt(short, long, help=WIDTH_HELP)]
    width: Option<i32>,

    #[structopt(parse(from_os_str), help=INFILE_HELP)]
    infile: PathBuf,
}

fn main() {
    let args = Cli::from_args();

    let script = read_script(&args.infile).unwrap_or_else(|e| die(e));

    let mut interpreter = Interpreter::new(script, args.input).unwrap_or_else(|err| die(err));
    while interpreter.next().is_some() {
        thread::sleep(Duration::from_millis(args.delay));
        interpreter.tape.print(args.ascii_only);
    }
}
