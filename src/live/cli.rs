use std::{
    error::Error,
    path::PathBuf,
};

use structopt::StructOpt;

use crate::{
    subcmd::SubCmd,
    util::is_valid_infile,
};

use super::Live;

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
    fn run(self) -> Result<(), Box<dyn Error>> {
        Live::new(self.ascii_values, self.infile).run();
        Ok(())
    }
}
