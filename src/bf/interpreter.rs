use std::{
    io::{Read, Write},
    usize,
};

use crate::bf::executor::Executor;

pub struct Interpreter<R, W> {
    executor: Executor<R, W>,
    code: Vec<u8>,
    instr_ptr: usize,
    bracket_map: Vec<usize>,
}

impl<R: Read, W: Write> Interpreter<R, W> {
    const NON_JMP_CHARS: &'static [u8] = b"><+-.,";
    const ALL_CHARS: &'static [u8] = b"><+-.,[]";

    fn parse_code(&mut self) -> Result<(), String> {
        let mut stk = Vec::<usize>::with_capacity(self.code.len());
        for (i, &b) in self.code.iter().enumerate() {
            if b == b'[' {
                stk.push(i);
            } else if b == b']' {
                if let Some(open_idx) = stk.pop() {
                    self.bracket_map[open_idx] = i;
                    self.bracket_map[i] = open_idx;
                } else {
                    return Err(format!("Unmatched closing bracket at position {}", i));
                }
            } else if !Self::NON_JMP_CHARS.contains(&b) {
                if !b.is_ascii() {
                    return Err(format!("Invalid character {} at position {}", b, i));
                }
            }
        }

        if !stk.is_empty() {
            return Err(format!(
                "Unmatched opening bracket at position {}",
                stk.pop().unwrap() // `is_empty` check above guarantees this won't panic
            ));
        }
        Ok(())
    }

    pub fn try_new(executor: Executor<R, W>, code: Vec<u8>) -> Result<Self, String> {
        let mut interpreter = Interpreter {
            bracket_map: vec![usize::MAX; code.len()],
            executor: executor,
            code: code
                .into_iter()
                .filter(|&b| Self::ALL_CHARS.contains(&b))
                .collect(),
            instr_ptr: 0,
        };
        interpreter.parse_code()?;
        Ok(interpreter)
    }

    pub fn exec_once(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.instr_ptr >= self.code.len() {
            return Err(format!(
                "Instruction pointer out of bounds: {} >= {}",
                self.instr_ptr,
                self.code.len()
            )
            .into());
        }

        match self.code[self.instr_ptr] {
            b'<' => self.executor.move_data_ptr(-1)?,
            b'>' => self.executor.move_data_ptr(1)?,
            b'+' => self.executor.add_cur_byte(1),
            b'-' => self.executor.sub_cur_byte(1),
            b'.' => self.executor.print_byte()?,
            b',' => self.executor.read_byte()?,
            b'[' => {
                if self.executor.cur_byte() == 0 {
                    self.instr_ptr = self.bracket_map[self.instr_ptr];
                }
            }
            b']' => {
                if self.executor.cur_byte() != 0 {
                    self.instr_ptr = self.bracket_map[self.instr_ptr];
                }
            }
            _ => unreachable!(), // `parse_code` guarantees this won't happen
        };
        self.instr_ptr += 1;
        Ok(())
    }

    pub fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.instr_ptr < self.code.len() {
            self.exec_once()?;
        }
        Ok(())
    }
}
