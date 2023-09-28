use std::{fs::File, io::Write, thread::sleep, time::Duration};

use anyhow::{Context, Result};

use crate::{
    err_file_open, err_file_write,
    interpreter::Interpreter,
    util::{common::get_width, read::read_script},
};

mod cli;
pub use cli::RunCli;

mod print;
use print::Printer;

fn run_subcmd(args: RunCli) -> Result<()> {
    let script = read_script(args.infile.as_ref())?
        .iter()
        .flat_map(|l| l.as_bytes())
        .copied()
        .collect::<Vec<_>>();

    let width = get_width(args.width);

    let mut interpreter =
        Interpreter::new(script, args.input.into_bytes(), None);

    let mut printer = Printer::new();

    if args.show_tape {
        printer.print(
            &interpreter
                .tape
                .chunks(width, args.ascii_values)
                .display(""),
        )?;
    }

    while let Some(frame) = interpreter.next() {
        if let Err(err) = frame {
            printer.print("Error: ")?;
            printer.print(&err.to_string())?;
            printer.print("\n")?;
            return Ok(());
        }

        printer.reset()?;
        if args.show_tape {
            sleep(Duration::from_millis(args.delay));
            printer.print(
                &interpreter
                    .tape
                    .chunks(width, args.ascii_values)
                    .display(""),
            )?;
        }
        printer.print(&interpreter.output())?;
    }

    if let Some(path) = args.outfile {
        File::create(&path)
            .with_context(|| err_file_open!(path))?
            .write_all(interpreter.output().as_bytes())
            .with_context(|| err_file_write!(path))?;
    }

    Ok(())
}
