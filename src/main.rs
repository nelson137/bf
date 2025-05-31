use std::process::exit;

use structopt::StructOpt;

#[cfg(feature="debug-server")]
#[macro_use]
mod debug_server;

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
use util::{err::BfResult, subcmd::SubCmd};

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
    #[cfg(feature="debug-server")]
    {
        use nix::{
            Error as NixError,
            errno::Errno,
            fcntl::{OFlag, open},
            sys::stat::Mode,
            unistd::mkfifo,
        };
        use debug_server::{PIPE_PATH, PIPE_FD};

        let mode = Mode::from_bits(0o664)
            .ok_or(bf_err!("failed to construct mode"))?;

        match mkfifo(PIPE_PATH, mode) {
            Ok(()) => (),
            Err(NixError::Sys(Errno::EEXIST)) => (),
            Err(e) => panic!("{}", e),
        };

        eprintln!("Blocking until pipe is opened for reading...");
        let fd = open(PIPE_PATH, OFlag::O_WRONLY, mode)
            .map_err(|e| format!("failed to open pipe for writing: {}", e))?;
        unsafe { PIPE_FD = Some(fd); }
    }

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
