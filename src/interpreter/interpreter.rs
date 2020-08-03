use std::collections::{HashMap, VecDeque};
use std::io::{self, Read};

use crate::util::die;

use super::tape::Tape;

#[derive(Debug)]
pub struct Interpreter {
    instructions: Vec<u8>,
    bracemap: HashMap<usize, usize>,
    ip: usize,
    pub tape: Tape,
    input: VecDeque<char>,
    pub output: String,
}

impl Interpreter {
    pub fn new<C: AsRef<[u8]>>(code: C, input: &str) -> Self {
        let instructions = Self::sanitize(code.as_ref());
        let bracemap = Self::build_bracemap(&instructions);
        Self {
            instructions,
            bracemap,
            ip: 0,
            tape: Tape::new(),
            input: input.chars().collect(),
            output: String::new(),
        }
    }

    fn is_instruction(c: &u8) -> bool {
        match *c as char {
            '+' | '-' | '>' | '<' | '[' | ']' | '.' | ',' => true,
            _ => false,
        }
    }

    fn sanitize(code: &[u8]) -> Vec<u8> {
        code.iter().cloned().filter(Self::is_instruction).collect()
    }

    fn build_bracemap(instructions: &[u8]) -> HashMap<usize, usize> {
        let mut open_brackets = Vec::new();
        let mut bracemap = HashMap::new();

        for (i, v) in instructions.iter().map(|i| *i as char).enumerate() {
            if v == '[' {
                open_brackets.push(i);
            } else if v == ']' {
                if let Some(open_i) = open_brackets.pop() {
                    bracemap.insert(open_i, i);
                    bracemap.insert(i, open_i);
                }
            }
        }

        bracemap
    }

    fn jump_bracket(&self) -> Result<usize, String> {
        match self.bracemap.get(&self.ip) {
            Some(next) => {
                let (begin, end) = if self.ip < *next {
                    (self.ip + 1, *next)
                } else {
                    (*next + 1, self.ip)
                };
                let loop_body = self.instructions.get(begin..end).unwrap();
                // If loop body contains '+' or '-'
                if loop_body.iter().any(|c| *c == 43 || *c == 45) {
                    Ok(next + 1)
                } else {
                    Err("infinite loop".to_string())
                }
            }
            None => Err("mismatched brackets".to_string()),
        }
    }

    fn read_char(&mut self) -> char {
        match self.input.pop_front() {
            Some(c) => c,
            None => {
                // Read one character from stdin
                let mut buf = [0u8; 1];
                if let Err(err) = io::stdin().read_exact(&mut buf) {
                    die(format!(
                        "failed to read character from stdin: {}",
                        err
                    ));
                }
                buf[0] as char
            }
        }
    }
}

impl Iterator for Interpreter {
    type Item = Result<char, String>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.instructions.is_empty()
            || self.ip > self.instructions.len() - 1
        {
            // No instructions or end of program
            return None;
        }

        let ins = self.instructions[self.ip] as char;
        let mut next_ip = Ok(self.ip + 1);

        match ins {
            '+' => self.tape.current().inc(),
            '-' => self.tape.current().dec(),
            '>' => self.tape.right(),
            '<' => self.tape.left(),
            '[' => {
                // Always check for mismatched bracket error
                let ni = self.jump_bracket();
                if let Err(err) = ni {
                    return Some(Err(err));
                } else if self.tape.current().value() == 0 {
                    next_ip = ni;
                }
            }
            ']' => {
                // Always check for mismatched bracket error
                let ni = self.jump_bracket();
                if let Err(err) = ni {
                    return Some(Err(err));
                } else if self.tape.current().value() != 0 {
                    next_ip = ni;
                }
            }
            '.' => self.output.push(self.tape.current().ascii()),
            ',' => {
                let c = self.read_char();
                self.tape.current().set(c);
            }
            _ => return None,
        }

        if let Ok(ip) = next_ip {
            self.ip = ip;
        }
        Some(next_ip.map(|_| ins))
    }
}
