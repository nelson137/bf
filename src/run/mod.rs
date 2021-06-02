use std::{fs::File, io::Write, thread::sleep, time::Duration};

use crate::{
    interpreter::Interpreter,
    util::{
        common::get_width,
        err::{err, BfResult},
        read::read_script,
    },
};

mod cli;
pub use cli::RunCli;

mod print;
use print::Printer;

fn run_subcmd(args: RunCli) -> BfResult<()> {
    let script = read_script(&args.infile)?;

    let width = get_width(args.width);

    let mut interpreter = Interpreter::new(script, args.input.into_bytes());

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
            .map_err(|e| err!(FileOpen, e, path.clone()))?
            .write_all(interpreter.output().as_bytes())
            .map_err(|e| err!(FileWrite, e, path))?;
    }

    Ok(())
}
