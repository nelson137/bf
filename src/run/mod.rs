use std::{
    error::Error,
    fs::File,
    io::Write,
    thread::sleep,
    time::Duration,
};

use crate::interpreter::Interpreter;
use crate::read::read_script;
use crate::util::get_width;

mod cli;
pub use cli::RunCli;

mod print;
use print::Printer;

fn run_subcmd(args: RunCli) -> Result<(), Box<dyn Error>> {
    let script = read_script(&args.infile)?;

    let width = get_width(args.width);

    let mut interpreter = Interpreter::new(script, &args.input);

    let mut printer = Printer::new();

    if args.show_tape {
        printer.print(
            &interpreter
                .tape
                .chunks(width)
                .display("", args.ascii_values),
        );
    }

    while let Some(frame) = interpreter.next() {
        if let Err(err) = frame {
            printer.print("Error: ");
            printer.print(&err);
            printer.print("\n");
            return Ok(());
        }

        printer.reset();
        if args.show_tape {
            sleep(Duration::from_millis(args.delay));
            printer.print(
                &interpreter
                    .tape
                    .chunks(width)
                    .display("", args.ascii_values),
            );
        }
        printer.print(&interpreter.output);
    }

    if let Some(path) = args.outfile {
        File::create(path)?
            .write_all(interpreter.output.as_bytes())?;
    }

    Ok(())
}
