#![feature(return_position_impl_trait_in_trait)]

use anyhow::Result;
use structopt::StructOpt;

mod commands;

mod interpreter;
#[macro_use]
mod util;
use util::cli::SubCmd;

mod widgets;

#[derive(Debug, StructOpt)]
enum Cli {
    Run(commands::run::RunCli),

    #[structopt(alias = "gen")]
    Generate(commands::generate::GenerateCli),

    Live(commands::live::LiveCli),

    InputDebug(commands::input_debug::InputDebugCli),
}

impl Cli {
    fn run_subcmd(self) -> Result<()> {
        match Self::from_args() {
            Self::Run(cli) => cli.run(),
            Self::Generate(cli) => cli.run(),
            Self::Live(cli) => cli.run(),
            Self::InputDebug(cli) => cli.run(),
        }
    }
}

fn main() -> Result<()> {
    #[cfg(windows)]
    if ansi_term::enable_ansi_support().is_err() {
        bail!("failed to enable ANSI support");
    }

    Cli::from_args().run_subcmd()
}
