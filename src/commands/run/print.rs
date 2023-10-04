use std::io::{self, Write};

use anyhow::{Context, Result};

use crate::{err_print, util::common::EOL};

pub struct Printer {
    writer: Box<dyn Write>,
    lines_printed: usize,
    has_final_eol: bool,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            writer: Box::new(io::stdout()),
            lines_printed: 0,
            has_final_eol: true,
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        // Go back to top of output, clearing each line
        while self.lines_printed > 0 {
            if self.has_final_eol {
                // Go up one line
                self.writer
                    .write_all(b"\x1b[1A")
                    .with_context(|| err_print!())?;
            } else {
                // All lines printed before the last have an EOL by definition
                self.has_final_eol = true;
            }

            // Clear line
            self.writer
                .write_all(b"\r\x1b[K")
                .with_context(|| err_print!())?;

            self.lines_printed -= 1;
        }

        Ok(())
    }

    pub fn print(&mut self, data: &str) -> Result<()> {
        // Detect if last line has EOL
        let has_final_eol = data.ends_with(EOL);

        // Print data
        let lines: Vec<_> = data.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            self.writer
                .write_all(line.as_bytes())
                .with_context(|| err_print!())?;
            if i < lines.len() - 1 || has_final_eol {
                self.writer
                    .write_all(EOL.as_bytes())
                    .with_context(|| err_print!())?;
            }
            self.lines_printed += 1;
        }

        // Only update variable if some data was printed (data != "")
        if !lines.is_empty() {
            self.has_final_eol = has_final_eol;
        }

        self.writer.flush().with_context(|| err_print!())?;

        Ok(())
    }
}