#![feature(return_position_impl_trait_in_trait)]

use anyhow::Result;
use structopt::StructOpt;

mod generate;
use generate::GenerateCli;

mod input_debug;
use input_debug::InputDebugCli;

mod interpreter;

mod live;
use live::LiveCli;

mod run;
use run::RunCli;

#[macro_use]
mod util;
use util::subcmd::SubCmd;

#[derive(Debug, StructOpt)]
enum Cli {
    Run(RunCli),

    #[structopt(alias = "gen")]
    Generate(GenerateCli),

    Live(LiveCli),

    InputDebug(InputDebugCli),
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
