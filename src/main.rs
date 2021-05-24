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

mod tui_util;

mod ui;

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

fn bf_main() -> BfResult<()> {
    #[cfg(windows)]
    if ansi_term::enable_ansi_support().is_err() {
        return Err(err!("failed to enable ANSI support"));
    }

    use Cli::*;
    match Cli::from_args() {
        Run(cli) => cli.run(),
        Generate(cli) => cli.run(),
        Live(cli) => cli.run(),
        InputDebug(cli) => cli.run(),
    }
}

fn main() {
    if let Err(err) = bf_main() {
        eprintln!("error: {}", err);
        exit(1);
    } else {
        exit(0);
    }
}
