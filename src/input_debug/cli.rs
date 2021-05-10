use std::{
    io::{Write, stdout},
};

use structopt::StructOpt;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
        disable_raw_mode,
        enable_raw_mode
    },
};

use crate::{
    subcmd::SubCmd,
    util::die,
};

use super::app::run;

const ABOUT: &str = "Live scripting playground";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct InputDebugCli {
}

impl SubCmd for InputDebugCli {
    fn run(self) {
        enable_raw_mode().unwrap_or_else(|e| die(e.to_string()));
        execute!(stdout(), EnableMouseCapture, EnterAlternateScreen)
            .unwrap_or_else(|e| die(e.to_string()));

        run();

        execute!(stdout(), DisableMouseCapture, LeaveAlternateScreen)
            .unwrap_or_else(|e| die(e.to_string()));
        disable_raw_mode().unwrap_or_else(|e| die(e.to_string()));
    }
}
