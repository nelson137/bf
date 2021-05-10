use structopt::StructOpt;

use crate::{
    subcmd::SubCmd,
    util::die,
};

use super::app::App;

const ABOUT: &str = "User input debugger";
const MOUSE_HELP: &str = "Whether to show mouse events";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct InputDebugCli {
    #[structopt(short="m", long, help=MOUSE_HELP)]
    enable_mouse: bool,
}

impl SubCmd for InputDebugCli {
    fn run(self) {
        App::new(self.enable_mouse)
            .unwrap_or_else(|e| die(e.to_string()))
            .run()
            .unwrap_or_else(|e| die(e.to_string()));
    }
}
