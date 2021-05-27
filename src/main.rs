use std::process::exit;

use structopt::StructOpt;

mod generate;
use generate::GenerateCli;

mod input_debug;
use input_debug::InputDebugCli;

mod interpreter;

mod live;
use live::LiveCli;

mod read;

mod run;
use run::RunCli;

mod subcmd;
use subcmd::SubCmd;

mod sync_util;

mod tui_util;

#[macro_use]
mod util;
use util::BfResult;

#[derive(Debug, StructOpt)]
enum Cli {
    Run(RunCli),

    #[structopt(alias = "gen")]
    Generate(GenerateCli),

    Live(LiveCli),

    InputDebug(InputDebugCli),
}

impl Cli {
    fn run_subcmd(self) -> BfResult<()> {
        match Self::from_args() {
            Self::Run(cli) => cli.run(),
            Self::Generate(cli) => cli.run(),
            Self::Live(cli) => cli.run(),
            Self::InputDebug(cli) => cli.run(),
        }
    }
}

fn bf_main() -> BfResult<()> {
    #[cfg(windows)]
    if ansi_term::enable_ansi_support().is_err() {
        return Err(err!("failed to enable ANSI support"));
    }

    Cli::from_args().run_subcmd()
}

fn main() {
    if let Err(err) = bf_main() {
        eprintln!("error: {}", err);
        exit(1);
    } else {
        exit(0);
    }
}
