use structopt::StructOpt;

use crate::{
    subcmd::SubCmd,
    util::die,
};

use super::app::App;

const ABOUT: &str = "Live scripting playground";

#[derive(Debug, StructOpt)]
#[structopt(about=ABOUT)]
pub struct InputDebugCli {
}

impl SubCmd for InputDebugCli {
    fn run(self) {
        App::new()
            .unwrap_or_else(|e| die(e.to_string()))
            .run()
            .unwrap_or_else(|e| die(e.to_string()));
    }
}
