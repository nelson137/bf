use structopt::StructOpt;

use crate::{subcmd::SubCmd, util::BfResult};

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
    fn run(self) -> BfResult<()> {
        App::new(self).and_then(|mut app| app.run())
    }
}
