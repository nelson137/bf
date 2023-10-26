use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use crate::cli::{parse_infile, ClapError, SubCmd};

#[derive(Debug, Parser)]
pub struct InlineScrollCli {
    #[arg(
        short,
        long,
        default_value="500",
        value_parser=parse_delay,
        hide_default_value=true,
    )]
    pub delay: u64,

    #[arg(value_parser=parse_infile)]
    pub infile: Option<PathBuf>,
}

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

impl SubCmd for InlineScrollCli {
    fn run(self) -> Result<()> {
        super::run(self)
    }
}
