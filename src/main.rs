use structopt::StructOpt;

mod generate;
use generate::GenerateCli;

mod interpreter;

mod live;
use live::LiveCli;

mod read;

mod run;
use run::RunCli;

mod subcmd;
use subcmd::SubCmd;

mod ui;

mod util;

#[derive(Debug, StructOpt)]
enum Cli {
    Run(RunCli),

    #[structopt(alias = "gen")]
    Generate(GenerateCli),

    Live(LiveCli),
}

fn main() {
    #[cfg(windows)]
    if let Err(_code) = ansi_term::enable_ansi_support() {
        eprintln!("Warning: ANSI support not enabled");
    }

    use Cli::*;
    match Cli::from_args() {
        Run(cli) => cli.run(),
        Generate(cli) => cli.run(),
        Live(cli) => cli.run(),
    }
}
