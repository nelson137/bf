use std::{fs::File, io::Write, path::PathBuf, thread::sleep, time::Duration};

use anyhow::{Context, Result};

use crate::{
    err_file_open, err_file_write,
    interpreter::Interpreter,
    util::{common::get_width, read::read_script},
};

use super::{print::Printer, RunCli};

pub struct App {
    delay: u64,
    show_tape: bool,
    width: i32,
    ascii_values: bool,
    outfile: Option<PathBuf>,
    interpreter: Interpreter,
    printer: Printer,
}

impl App {
    pub fn new(cli: RunCli) -> Result<Self> {
        let script_lines = read_script(cli.infile.as_ref())?;
        let script = script_lines.iter().flat_map(|l| l.as_bytes()).copied();

        let input = cli.input.into_bytes().into_iter().collect();

        Ok(Self {
            delay: cli.delay,
            show_tape: cli.show_tape,
            width: get_width(cli.width),
            ascii_values: cli.ascii_values,
            outfile: cli.outfile,
            interpreter: Interpreter::new(script, input, None),
            printer: Printer::new(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        if self.show_tape {
            self.print_tape()?;
        }

        while let Some(frame) = self.interpreter.next() {
            if let Err(err) = frame {
                self.printer.print("Error: ")?;
                self.printer.print(&err.to_string())?;
                self.printer.print("\n")?;
                return Ok(());
            }

            self.printer.reset()?;
            if self.show_tape {
                sleep(Duration::from_millis(self.delay));
                self.print_tape()?;
            }
            self.printer.print(&self.interpreter.output())?;
        }

        if let Some(path) = &self.outfile {
            File::create(path)
                .with_context(|| err_file_open!(path))?
                .write_all(self.interpreter.output().as_bytes())
                .with_context(|| err_file_write!(path))?;
        }

        Ok(())
    }

    fn print_tape(&mut self) -> Result<()> {
        let tape_display = self
            .interpreter
            .tape
            .chunks(self.width, self.ascii_values)
            .display("");
        self.printer.print(&tape_display)
    }
}
