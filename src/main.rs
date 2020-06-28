use std::path::PathBuf;

use structopt::StructOpt;

const DELAY_HELP: &str = "The delay, in milliseconds, between the evaluation\
                          of each Brainfuck instruction.";
const INPUT_HELP: &str = "The input to provide the Brainfuck program for the\
                          read (,) instruction.";
const DUMP_HELP: &str = "Print the final state of the tape after execution.";
const SHOW_HELP: &str = "Show the tape during execution. Use -d,--delay to\
                         slow down execution.";
const WIDTH_HELP: &str = "The maximum width of the terminal for formatting the\
                          tape output.";
const INFILE_HELP: &str = "The path to the Brainfuck script to execute. Can be\
                           a hyphen (-) to read the script from stdin.";

#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(short, long, default_value="0", help=DELAY_HELP)]
    delay: i32,

    #[structopt(short, long, default_value="", help=INPUT_HELP)]
    input: String,

    #[structopt(long, help=DUMP_HELP)]
    dump_tape: bool,

    #[structopt(long, help=SHOW_HELP)]
    show_tape: bool,

    #[structopt(short, long, help=WIDTH_HELP)]
    width: Option<i32>,

    #[structopt(parse(from_os_str), help=INFILE_HELP)]
    infile: PathBuf,
}

fn main() {
    let args = Cli::from_args();
    dbg!(args);
}
