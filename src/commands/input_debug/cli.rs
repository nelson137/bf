use anyhow::Result;
use clap::Parser;

use crate::util::cli::SubCmd;

use super::app::App;

const ABOUT: &str = "User input debugger";
const MOUSE_HELP: &str = "Whether to show mouse events";

#[derive(Debug, Parser)]
#[command(about=ABOUT)]
pub struct InputDebugCli {
    #[arg(short='m', long, help=MOUSE_HELP)]
    pub enable_mouse: bool,
}

impl SubCmd for InputDebugCli {
    fn run(self) -> Result<()> {
        App::new(self).and_then(|mut app| app.run())
    }
}
