use std::path::PathBuf;

use anyhow::Result;
use structopt::StructOpt;

use crate::util::cli::{is_valid_infile, SubCmd};

use super::app::App;

const ABOUT: &str = "Live scripting playground";
const ASCII_HELP: &str = "Show the ASCII characters in the tape output \
                          instead of the decimal values";
const INFILE_HELP: &str = "The script to edit in live mode";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct LiveCli {
    #[structopt(short, long, help=ASCII_HELP)]
    pub ascii_values: bool,

    #[structopt(validator=is_valid_infile, help=INFILE_HELP)]
    pub infile: Option<PathBuf>,
}

impl SubCmd for LiveCli {
    fn run(self) -> Result<()> {
        App::new(self).and_then(|mut app| app.run())
    }
}
