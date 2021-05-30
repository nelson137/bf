use std::path::PathBuf;

use structopt::StructOpt;

use crate::util::{
    err::BfResult,
    subcmd::SubCmd,
    common::{is_valid_infile, is_valid_width},
};

use super::run_subcmd;

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
const INFILE_HELP: &str = "The path to the Brainfuck script to execute. Read \
                           from stdin if infile is a dash (-) or is omitted.";
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
    pub delay: u64,

    #[structopt(
        short,
        long,
        default_value="",
        hide_default_value=true,
        help=INPUT_HELP
    )]
    pub input: String,

    #[structopt(short, long, help=SHOW_HELP)]
    pub show_tape: bool,

    #[structopt(short, long, validator=is_valid_width, help=WIDTH_HELP)]
    pub width: Option<usize>,

    #[structopt(short, long, help=ASCII_HELP)]
    pub ascii_values: bool,

    #[structopt(short, long, help=OUTFILE_HELP)]
    pub outfile: Option<PathBuf>,

    #[structopt(validator=is_valid_infile, help=INFILE_HELP)]
    pub infile: Option<PathBuf>,
}

impl SubCmd for RunCli {
    fn run(self) -> BfResult<()> {
        run_subcmd(self)
    }
}
