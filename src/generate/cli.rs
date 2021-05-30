use std::path::PathBuf;

use structopt::StructOpt;

use crate::util::{err::BfResult, subcmd::SubCmd};

use super::subcmd_generate;

const ABOUT: &str =
    "Generate a Brainfuck script that prints the given text file";
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

const GEN_MODES: &[&str] = &["charwise", "linewise", "unique-chars"];

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct GenerateCli {
    #[structopt(short, long, help=NEWLINE_HELP)]
    pub newline: bool,

    #[structopt(short, long, help=OUTFILE_HELP)]
    pub outfile: Option<PathBuf>,

    #[structopt(
        possible_values=GEN_MODES,
        help=MODE_HELP,
        long_help=MODE_HELP_LONG
    )]
    pub mode: String,

    #[structopt(help=INFILE_HELP)]
    pub infile: Option<PathBuf>,
}

impl SubCmd for GenerateCli {
    fn run(self) -> BfResult<()> {
        subcmd_generate(self)
    }
}
