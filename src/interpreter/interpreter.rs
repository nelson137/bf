use std::{
    collections::{HashMap, VecDeque},
    io::{self, Read},
};

use crate::util::err::BfResult;

use super::tape::Tape;

#[derive(Debug)]
pub struct Interpreter {
    instructions: Vec<u8>,
    bracemap: HashMap<usize, usize>,
    ip: usize,
    pub tape: Tape,
    pub input: VecDeque<u8>,
    pub output: Vec<u8>,
}

impl Interpreter {
    pub fn new<I: AsRef<[u8]>>(code: I, input: I) -> Self {
        let instructions = Self::sanitize(code.as_ref());
        let bracemap = Self::build_bracemap(&instructions);
        Self {
            instructions,
            bracemap,
            ip: 0,
            tape: Tape::default(),
            input: input.as_ref().iter().cloned().collect(),
            output: Vec::new(),
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

    fn jump_bracket(&self) -> BfResult<usize> {
        match self.bracemap.get(&self.ip) {
            Some(next) => Ok(next + 1),
            None => Err("mismatched brackets".into()),
        }
    }

    fn read_char(&mut self) -> BfResult<u8> {
        Ok(match self.input.pop_front() {
            Some(c) => Ok(c),
            None => {
                // Read one character from stdin
                let mut buf = [0u8; 1];
                match io::stdin().read_exact(&mut buf) {
                    Ok(_) => Ok(buf[0]),
                    Err(e) => Err(format!(
                        "failed to read character from stdin: {}",
                        e
                    )),
                }
            }
        }?)
    }

    pub fn peek_instruction(&self) -> Option<char> {
        if self.instructions.is_empty()
            || self.ip > self.instructions.len() - 1
        {
            None
        } else {
            Some(self.instructions[self.ip] as char)
        }
    }

    pub fn output(&self) -> String {
        String::from_utf8_lossy(&self.output).into_owned()
    }
}

impl Iterator for Interpreter {
    type Item = BfResult<char>;
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
                    return Some(Err(err.into()));
                } else if self.tape.current().value() == 0 {
                    next_ip = ni;
                }
            }
            ']' => {
                // Always check for mismatched bracket error
                let ni = self.jump_bracket();
                if let Err(err) = ni {
                    return Some(Err(err.into()));
                } else if self.tape.current().value() != 0 {
                    next_ip = ni;
                }
            }
            '.' => self.output.push(self.tape.current().value()),
            ',' => match self.read_char() {
                Ok(c) => self.tape.current().set(c),
                Err(e) => return Some(Err(e)),
            },
            _ => return None,
        }

        if let Ok(ip) = next_ip {
            self.ip = ip;
        }
        Some(next_ip.map(|_| ins))
    }
}
