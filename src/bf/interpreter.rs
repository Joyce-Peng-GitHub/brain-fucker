use std::{
    io::{Read, Write},
    usize,
};

use crate::bf::executor::Executor;

/// (character, number of occurrences or index of the matching bracket)
struct Code(u8, usize);

pub struct Interpreter<R, W> {
    executor: Executor<R, W>,
    codes: Vec<Code>,
    instr_ptr: usize,
}

impl<R: Read, W: Write> Interpreter<R, W> {
    const VALID_CHARS: &'static [u8] = b"><+-.,[]";
    const INC_DATA_PTR: u8 = b'>';
    const DEC_DATA_PTR: u8 = b'<';
    const INC_CUR_BYTE: u8 = b'+';
    const DEC_CUR_BYTE: u8 = b'-';
    const WRITE_BYTE: u8 = b'.';
    const READ_BYTE: u8 = b',';
    const JMP_FWD: u8 = b'[';
    const JMP_BWD: u8 = b']';

    fn parse_code(code: &Vec<u8>) -> Result<Vec<Code>, String> {
        let mut codes = Vec::<Code>::with_capacity(code.len());

        for &b in code.iter() {
            if !Self::VALID_CHARS.contains(&b) {
                continue;
            }

            if !codes.is_empty()
                && codes.last().unwrap().0 == b
                && b != Self::JMP_FWD
                && b != Self::JMP_BWD
            {
                codes.last_mut().unwrap().1 += 1;
            } else {
                codes.push(Code(b, 1));
            }
        }

        let mut stk = Vec::<usize>::with_capacity(codes.len());
        for i in 0..codes.len() {
            match codes[i].0 {
                Self::JMP_FWD => stk.push(i),
                Self::JMP_BWD => {
                    if let Some(open_idx) = stk.pop() {
                        codes[open_idx].1 = i;
                        codes[i].1 = open_idx;
                    } else {
                        return Err(format!("Unmatched closing bracket at position {}", i));
                    }
                }
                _ => {}
            }
        }
        if !stk.is_empty() {
            return Err(format!(
                "Unmatched opening bracket at position {}",
                stk.pop().unwrap() // `is_empty` check above guarantees this won't panic
            ));
        }

        Ok(codes)
    }

    pub fn try_new(executor: Executor<R, W>, code: Vec<u8>) -> Result<Self, String> {
        let interpreter = Interpreter {
            executor: executor,
            codes: Self::parse_code(&code)?,
            instr_ptr: 0,
        };
        Ok(interpreter)
    }

    pub fn exec_once(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.instr_ptr >= self.codes.len() {
            return Err(format!(
                "Instruction pointer out of bounds: {} >= {}",
                self.instr_ptr,
                self.codes.len()
            )
            .into());
        }

        match self.codes[self.instr_ptr] {
            Code(Self::INC_DATA_PTR, cnt) => self.executor.move_data_ptr(cnt as isize)?,
            Code(Self::DEC_DATA_PTR, cnt) => self.executor.move_data_ptr(-(cnt as isize))?,
            Code(Self::INC_CUR_BYTE, cnt) => {
                self.executor.add_cur_byte((cnt & (u8::MAX as usize)) as u8)
            }
            Code(Self::DEC_CUR_BYTE, cnt) => {
                self.executor.sub_cur_byte((cnt & (u8::MAX as usize)) as u8)
            }
            Code(Self::WRITE_BYTE, cnt) => {
                for _ in 0..cnt {
                    self.executor.write_byte()?;
                }
            }
            Code(Self::READ_BYTE, cnt) => {
                for _ in 0..cnt {
                    self.executor.read_byte()?;
                }
            }
            Code(Self::JMP_FWD, match_idx) => {
                if self.executor.cur_byte() == 0 {
                    self.instr_ptr = match_idx;
                }
            }
            Code(Self::JMP_BWD, match_idx) => {
                if self.executor.cur_byte() != 0 {
                    self.instr_ptr = match_idx;
                }
            }
            _ => unreachable!(), // `parse_code` guarantees this won't happen
        }

        self.instr_ptr += 1;
        Ok(())
    }

    pub fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.instr_ptr < self.codes.len() {
            self.exec_once()?;
        }
        Ok(())
    }
}
