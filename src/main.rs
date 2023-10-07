#![feature(return_position_impl_trait_in_trait)]
#![feature(try_blocks)]

use anyhow::Result;
use clap::Parser;

mod commands;

mod interpreter;
#[macro_use]
mod util;
use util::cli::SubCmd;

mod widgets;

#[derive(Debug, Parser)]
enum Cli {
    Run(commands::run::RunCli),

    #[command(alias = "gen")]
    Generate(commands::generate::GenerateCli),

    Live(commands::live::LiveCli),

    InputDebug(commands::input_debug::InputDebugCli),

    InlineScroll(commands::inline_scroll::InlineScrollCli),
}

impl Cli {
    fn run_subcmd(self) -> Result<()> {
        match Self::parse() {
            Self::Run(cli) => cli.run(),
            Self::Generate(cli) => cli.run(),
            Self::Live(cli) => cli.run(),
            Self::InputDebug(cli) => cli.run(),
            Self::InlineScroll(cli) => cli.run(),
        }
    }
}

fn main() -> Result<()> {
    #[cfg(windows)]
    if ansi_term::enable_ansi_support().is_err() {
        bail!("failed to enable ANSI support");
    }

    Cli::parse().run_subcmd()
}
