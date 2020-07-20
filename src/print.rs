use std::io::{self, Write};

use crate::util::die;

fn ends_with_eol(s: &str) -> bool {
    s.ends_with('\n') || s.ends_with("\r\n")
}

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

    pub fn reset(&mut self) {
        // Go back to top of output, clearing each line
        while self.lines_printed > 0 {
            if self.has_final_eol {
                // Go up one line
                self.writer
                    .write_all(b"\x1b[1A")
                    .unwrap_or_else(|err| die(err.to_string()));
            } else {
                // All lines printed before the last have an EOL by definition
                self.has_final_eol = true;
            }

            // Clear line
            self.writer
                .write_all(b"\r\x1b[K")
                .unwrap_or_else(|err| die(err.to_string()));

            self.lines_printed -= 1;
        }
    }

    pub fn print(&mut self, data: String) {
        // Detect if last line has EOL
        let has_final_eol = ends_with_eol(&data);

        // Print data
        let lines: Vec<_> = data.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            self.writer
                .write_all(line.as_bytes())
                .unwrap_or_else(|err| die(err.to_string()));
            if i < lines.len() - 1 || has_final_eol {
                self.writer
                    .write_all(b"\n")
                    .unwrap_or_else(|err| die(err.to_string()));
            }
            self.lines_printed += 1;
        }

        // Only update variable if some data was printed (data != "")
        if !lines.is_empty() {
            self.has_final_eol = has_final_eol;
        }

        self.writer
            .flush()
            .unwrap_or_else(|err| die(err.to_string()));
    }
}
