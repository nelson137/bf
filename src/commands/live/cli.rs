use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::util::cli::{parse_infile, SubCmd};

use super::app::App;

const ABOUT: &str = "Live scripting playground";
const ASCII_HELP: &str = "Show the ASCII characters in the tape output \
                          instead of the decimal values";
const INFILE_HELP: &str = "The script to edit in live mode";

#[derive(Debug, Parser)]
#[command(about=ABOUT)]
pub struct LiveCli {
    #[arg(short, long, help=ASCII_HELP)]
    pub ascii_values: bool,

    #[arg(value_parser=parse_infile, help=INFILE_HELP)]
    pub infile: Option<PathBuf>,
}

impl SubCmd for LiveCli {
    fn run(self) -> Result<()> {
        App::new(self).and_then(|mut app| app.run())
    }
}
