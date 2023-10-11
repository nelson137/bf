use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::util::cli::{parse_infile, parse_width, ClapError, SubCmd};

use super::app::App;

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

fn parse_delay(value: &str) -> Result<u64, ClapError> {
    match value.parse::<i64>() {
        Ok(n) => {
            if n < 0 {
                Err("value must be an integer >= 0".into())
            } else {
                Ok(n as u64)
            }
        }
        Err(err) => Err(err.into()),
    }
}

#[derive(Debug, Parser)]
#[command(about=ABOUT)]
pub struct RunCli {
    #[arg(
        short,
        long,
        default_value="0",
        value_parser=parse_delay,
        hide_default_value=true,
        help=DELAY_HELP
    )]
    pub delay: u64,

    #[arg(
        short,
        long,
        default_value="",
        hide_default_value=true,
        help=INPUT_HELP
    )]
    pub input: String,

    #[arg(short, long, help=SHOW_HELP)]
    pub show_tape: bool,

    #[arg(short, long, value_parser=parse_width, help=WIDTH_HELP)]
    pub width: Option<usize>,

    #[arg(short, long, help=ASCII_HELP)]
    pub ascii_values: bool,

    #[arg(short, long, help=OUTFILE_HELP)]
    pub outfile: Option<PathBuf>,

    #[arg(value_parser=parse_infile, help=INFILE_HELP)]
    pub infile: Option<PathBuf>,
}

impl SubCmd for RunCli {
    fn run(self) -> Result<()> {
        App::new(self)?.run()
    }
}
