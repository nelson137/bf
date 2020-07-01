use std::io::{self, Write};

use crate::util::ends_with_eol;

pub struct Printer {
    lines_printed: usize,
    has_final_eol: bool,
}

impl Printer {
    pub fn new() -> Self {
        Self {
            lines_printed: 0,
            has_final_eol: true,
        }
    }

    pub fn reset(&mut self) {
        // Go back to top of output, clearing each line
        while self.lines_printed > 0 {
            if self.has_final_eol {
                // Go up one line
                print!("\x1b[1A");
            } else {
                // All lines printed before the last have an EOL by definition
                self.has_final_eol = true;
            }

            // Clear line
            print!("\r\x1b[K");

            self.lines_printed -= 1;
        }
    }

    pub fn print(&mut self, data: String) {
        // Detect if last line has EOL
        self.has_final_eol = ends_with_eol(&data);

        // Print data
        let lines: Vec<_> = data.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            print!("{}", line);
            if i < lines.len() - 1 || self.has_final_eol {
                println!();
            }
            self.lines_printed += 1;
        }

        io::stdout().flush().unwrap();
    }
}
