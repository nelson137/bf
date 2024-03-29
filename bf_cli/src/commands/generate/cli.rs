use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::cli::SubCmd;

use super::subcmd_generate;

const ABOUT: &str =
    "Generate a Brainfuck script that prints the given text file \
    (aliases: gen, g)";
const NEWLINE_HELP: &str =
    "Append a final newline to the data if it is missing.";
const OUTFILE_HELP: &str =
    "The file to which the generated script is written. If none is given \
print to stdout.";
const MODE_HELP: &str =
    "The method that the generated script will use to print the given data. \
    Use `--help` to see descriptions of each mode.";
const MODE_HELP_LONG: &str =
    "The method that the generated script will use to print the given data. \
    charwise (one loop): each byte in the data gets a cell in Brainfuck \
    memory; each cell is printed in order. linewise (one loop per line): \
    similar to charwise except lines are created in memory then printed one \
    at a time. unique-chars (one loop): each unique byte in the data gets a \
    cell in memory.";
const INFILE_HELP: &str =
    "The file that will be printed by the generated script. If none is given \
    read from stdin.";

const GEN_MODES: [&str; 3] = ["charwise", "linewise", "unique-chars"];

#[derive(Debug, Parser)]
#[structopt(about=ABOUT)]
pub struct GenerateCli {
    #[arg(short, long, help=NEWLINE_HELP)]
    pub newline: bool,

    #[arg(short, long, help=OUTFILE_HELP)]
    pub outfile: Option<PathBuf>,

    #[arg(value_parser=GEN_MODES, help=MODE_HELP, long_help=MODE_HELP_LONG)]
    pub mode: String,

    #[arg(help=INFILE_HELP)]
    pub infile: Option<PathBuf>,
}

impl SubCmd for GenerateCli {
    fn run(self) -> Result<()> {
        subcmd_generate(self)
    }
}
