use structopt::StructOpt;

mod generate;
use generate::GenerateCli;

mod interpreter;

mod live;
use live::LiveCli;

mod input_debug;
use input_debug::InputDebugCli;

mod read;

mod run;
use run::RunCli;

mod subcmd;
use subcmd::SubCmd;

mod tui_util;

mod ui;

mod util;
use util::die;

#[derive(Debug, StructOpt)]
enum Cli {
    Run(RunCli),

    #[structopt(alias = "gen")]
    Generate(GenerateCli),

    Live(LiveCli),

    InputDebug(InputDebugCli),
}

fn main() {
    #[cfg(windows)]
    if ansi_term::enable_ansi_support().is_err() {
        die("failed to enable ANSI support".to_string());
    }

    use Cli::*;
    match Cli::from_args() {
        Run(cli) => cli.run(),
        Generate(cli) => cli.run(),
        Live(cli) => cli.run(),
        InputDebug(cli) => cli.run(),
    }
}
