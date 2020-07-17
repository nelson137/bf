use structopt::StructOpt;

mod run;
use run::RunCli;

mod subcmd;
use subcmd::SubCmd;

mod util;

#[derive(Debug, StructOpt)]
enum Cli {
    Run(RunCli),
}

fn main() {
    #[cfg(windows)]
    if let Err(_code) = ansi_term::enable_ansi_support() {
        eprintln!("Warning: ANSI support not enabled");
    }

    match Cli::from_args() {
        Cli::Run(cli) => cli.run(),
    }
}
