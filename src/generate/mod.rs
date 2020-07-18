use std::path::PathBuf;
use std::str::FromStr;

use structopt::StructOpt;

use crate::subcmd::SubCmd;

#[derive(Debug)]
enum GenerateMode {
    Onto,
}

impl FromStr for GenerateMode {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, String> {
        use GenerateMode::*;
        match value {
            "onto" => Ok(Onto),
            _ => Err("must be one of: 'onto'".to_string()),
        }
    }
}

const ABOUT: &str =
    "Generate a Brainfuck script that prints the given text file";
const OUTFILE_HELP: &str = "";
const MODE_HELP: &str = "";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct GenerateCli {
    #[structopt(short, long, help=OUTFILE_HELP)]
    outfile: Option<PathBuf>,

    #[structopt(help=MODE_HELP)]
    mode: GenerateMode,
}

impl SubCmd for GenerateCli {
    fn run(self) {
        dbg!(self);
    }
}
