use std::fs::File;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use crate::interpreter::Interpreter;
use crate::read::read_script;
use crate::util::{die, get_width};

mod cli;
pub use cli::RunCli;

mod print;
use print::Printer;

fn run_subcmd(args: RunCli) {
    let script = read_script(&args.infile).unwrap_or_else(|e| die(e));

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
            return;
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
        File::create(path)
            .unwrap_or_else(|err| die(err.to_string()))
            .write_all(interpreter.output.as_bytes())
            .unwrap_or_else(|err| die(err.to_string()));
    }
}
