use std::{
    io::{Read, Write},
    usize,
};

use crate::bf::executor::Executor;

enum Instruction {
    MoveDataPtr(isize),
    WrappingAddCurByte(u8),
    WriteByte,
    ReadByte,
    JmpFwd(usize),
    JmpBwd(usize),
}

impl Instruction {
    const INC_DATA_PTR: u8 = b'>';
    const DEC_DATA_PTR: u8 = b'<';
    const INC_CUR_BYTE: u8 = b'+';
    const DEC_CUR_BYTE: u8 = b'-';
    const WRITE_BYTE: u8 = b'.';
    const READ_BYTE: u8 = b',';
    const JMP_FWD: u8 = b'[';
    const JMP_BWD: u8 = b']';

    fn from_byte(b: u8) -> Option<Self> {
        match b {
            Self::INC_DATA_PTR => Some(Self::MoveDataPtr(1)),
            Self::DEC_DATA_PTR => Some(Self::MoveDataPtr(-1)),
            Self::INC_CUR_BYTE => Some(Self::WrappingAddCurByte(1)),
            Self::DEC_CUR_BYTE => Some(Self::WrappingAddCurByte(u8::MAX)), // -1 mod 256
            Self::WRITE_BYTE => Some(Self::WriteByte),
            Self::READ_BYTE => Some(Self::ReadByte),
            Self::JMP_FWD => Some(Self::JmpFwd(usize::MAX)), // Placeholder, will be set in `parse_codes`
            Self::JMP_BWD => Some(Self::JmpBwd(usize::MAX)), // Placeholder, will be set in `parse_codes`
            _ => None,
        }
    }
}

pub struct Interpreter<R, W> {
    executor: Executor<R, W>,
    instructions: Vec<Instruction>,
    instr_ptr: usize,
}

impl<R: Read, W: Write> Interpreter<R, W> {
    fn compress_codes(codes: &Vec<u8>) -> Result<Vec<Instruction>, String> {
        let mut instructions = Vec::<Instruction>::with_capacity(codes.len());
        let mut stk = Vec::<usize>::with_capacity(codes.len());

        for &b in codes.iter() {
            if let Some(instr) = Instruction::from_byte(b) {
                if let Instruction::JmpFwd(pos) = instr {
                    debug_assert_eq!(pos, usize::MAX);
                    stk.push(instructions.len());
                    instructions.push(instr);
                    continue;
                } else if let Instruction::JmpBwd(pos) = instr {
                    debug_assert_eq!(pos, usize::MAX);
                    if let Some(open_pos) = stk.pop() {
                        let close_pos = instructions.len();
                        if let Instruction::JmpFwd(ref mut open_match_pos) = instructions[open_pos]
                        {
                            *open_match_pos = close_pos;
                        } else {
                            unreachable!();
                        }
                        instructions.push(Instruction::JmpBwd(open_pos));
                    } else {
                        return Err(format!(
                            "Unmatched closing bracket at position {}",
                            instructions.len()
                        ));
                    }
                    continue;
                }

                if instructions.is_empty()
                    || matches!(instr, Instruction::WriteByte | Instruction::ReadByte)
                    || std::mem::discriminant(instructions.last().unwrap())
                        != std::mem::discriminant(&instr)
                {
                    instructions.push(instr);
                    continue;
                }

                match instr {
                    Instruction::MoveDataPtr(offset) => {
                        if let Instruction::MoveDataPtr(cur_offset) =
                            instructions.last_mut().unwrap()
                        {
                            *cur_offset += offset;
                        } else {
                            unreachable!(); // The `if` condition above guarantees this won't happen
                        }
                    }
                    Instruction::WrappingAddCurByte(diff) => {
                        if let Instruction::WrappingAddCurByte(cur_diff) =
                            instructions.last_mut().unwrap()
                        {
                            *cur_diff = cur_diff.wrapping_add(diff);
                        } else {
                            unreachable!(); // The `if` condition above guarantees this won't happen
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        if !stk.is_empty() {
            return Err(format!(
                "Unmatched opening bracket at position {}",
                stk.pop().unwrap() // `is_empty` check above guarantees this won't panic
            ));
        }

        Ok(instructions)
    }

    pub fn try_new(executor: Executor<R, W>, codes: Vec<u8>) -> Result<Self, String> {
        let interpreter = Interpreter {
            executor: executor,
            instructions: Self::compress_codes(&codes)?,
            instr_ptr: 0,
        };
        Ok(interpreter)
    }

    pub fn exec_once(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.instr_ptr >= self.instructions.len() {
            return Err(format!(
                "Instruction pointer out of bounds: {} >= {}",
                self.instr_ptr,
                self.instructions.len()
            )
            .into());
        }

        match self.instructions[self.instr_ptr] {
            Instruction::MoveDataPtr(offset) => self.executor.move_data_ptr(offset)?,
            Instruction::WrappingAddCurByte(diff) => self.executor.add_cur_byte(diff),
            Instruction::WriteByte => self.executor.write_byte()?,
            Instruction::ReadByte => self.executor.read_byte()?,
            Instruction::JmpFwd(pos) => {
                if self.executor.cur_byte() == 0 {
                    self.instr_ptr = pos;
                }
            }
            Instruction::JmpBwd(pos) => {
                if self.executor.cur_byte() != 0 {
                    self.instr_ptr = pos;
                }
            }
        }

        self.instr_ptr += 1;
        Ok(())
    }

    pub fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.instr_ptr < self.instructions.len() {
            self.exec_once()?;
        }
        Ok(())
    }
}
