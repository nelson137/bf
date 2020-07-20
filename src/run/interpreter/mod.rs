use std::collections::{HashMap, VecDeque};
use std::io::{self, Read};

use crate::util::die;

mod cell;

mod tape;
use tape::Tape;

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
    pub fn new(code: Vec<u8>, input: String) -> Self {
        let instructions = Self::sanitize(code);
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
        ['+', '-', '>', '<', '[', ']', '.', ',']
            .iter()
            .map(|i| *i as u8)
            .any(|i| i == *c)
    }

    fn sanitize(code: Vec<u8>) -> Vec<u8> {
        code.into_iter().filter(Self::is_instruction).collect()
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

    fn jump_bracket(&self) -> usize {
        match self.bracemap.get(&self.ip) {
            Some(next) => next + 1,
            None => die("mismatched brackets".to_string()),
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
                        "Failed to read character from stdin: {}",
                        err
                    ));
                }
                buf[0] as char
            }
        }
    }
}

impl Iterator for Interpreter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.instructions.is_empty()
            || self.ip > self.instructions.len() - 1
        {
            // No instructions or end of program
            return None;
        }

        let ins = self.instructions[self.ip] as char;
        let mut next_ip = self.ip + 1;

        match ins {
            '+' => self.tape.current().inc(),
            '-' => self.tape.current().dec(),
            '>' => self.tape.right(),
            '<' => self.tape.left(),
            '[' => {
                if self.tape.current().value() == 0 {
                    next_ip = self.jump_bracket();
                }
            }
            ']' => {
                if self.tape.current().value() != 0 {
                    next_ip = self.jump_bracket();
                }
            }
            '.' => self.output.push(self.tape.current().ascii()),
            ',' => {
                let c = self.read_char();
                self.tape.current().set(c);
            }
            _ => return None,
        }

        self.ip = next_ip;
        Some(ins)
    }
}
