use anyhow::Result;
use structopt::StructOpt;

use crate::util::cli::SubCmd;

use super::app::App;

const ABOUT: &str = "User input debugger";
const MOUSE_HELP: &str = "Whether to show mouse events";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct InputDebugCli {
    #[structopt(short="m", long, help=MOUSE_HELP)]
    pub enable_mouse: bool,
}

impl SubCmd for InputDebugCli {
    fn run(self) -> Result<()> {
        App::new(self).and_then(|mut app| app.run())
    }
}
