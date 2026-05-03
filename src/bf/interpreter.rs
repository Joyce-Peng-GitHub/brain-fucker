use std::{
    io::{Read, Write},
    usize,
};

use crate::bf::executor::Executor;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BfInstr {
    MoveDataPtr(isize),
    WrappingAddCurByte(u8),
    WriteByte,
    ReadByte,
    JmpFwd(usize),
    JmpBwd(usize),
}

impl BfInstr {
    const INC_DATA_PTR: u8 = b'>';
    const DEC_DATA_PTR: u8 = b'<';
    const INC_CUR_BYTE: u8 = b'+';
    const DEC_CUR_BYTE: u8 = b'-';
    const WRITE_BYTE: u8 = b'.';
    const READ_BYTE: u8 = b',';
    const JMP_FWD: u8 = b'[';
    const JMP_BWD: u8 = b']';

    const JMP_POS_PLACEHOLDER: usize = usize::MAX;

    fn from_byte(b: u8) -> Option<Self> {
        match b {
            Self::INC_DATA_PTR => Some(Self::MoveDataPtr(1)),
            Self::DEC_DATA_PTR => Some(Self::MoveDataPtr(-1)),
            Self::INC_CUR_BYTE => Some(Self::WrappingAddCurByte(1)),
            Self::DEC_CUR_BYTE => Some(Self::WrappingAddCurByte(u8::MAX)), // -1 mod 256
            Self::WRITE_BYTE => Some(Self::WriteByte),
            Self::READ_BYTE => Some(Self::ReadByte),
            Self::JMP_FWD => Some(Self::JmpFwd(Self::JMP_POS_PLACEHOLDER)),
            Self::JMP_BWD => Some(Self::JmpBwd(Self::JMP_POS_PLACEHOLDER)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ExtInstr {
    MoveDataPtr(isize),
    WrappingAddCurByte(u8),
    WriteByte,
    ReadByte,
    JmpFwd(usize),
    JmpBwd(usize),
    ClearByte,
    FindZeroByte(isize), // step
}

impl ExtInstr {
    const JMP_POS_PLACEHOLDER: usize = BfInstr::JMP_POS_PLACEHOLDER;

    fn from_bf_instr(instr: &BfInstr) -> Self {
        match instr {
            &BfInstr::MoveDataPtr(offset) => Self::MoveDataPtr(offset),
            &BfInstr::WrappingAddCurByte(diff) => Self::WrappingAddCurByte(diff),
            &BfInstr::WriteByte => Self::WriteByte,
            &BfInstr::ReadByte => Self::ReadByte,
            &BfInstr::JmpFwd(pos) => Self::JmpFwd(pos),
            &BfInstr::JmpBwd(pos) => Self::JmpBwd(pos),
        }
    }
}

pub struct Interpreter<R, W> {
    executor: Executor<R, W>,
    instructions: Vec<ExtInstr>,
    instr_ptr: usize,
}

impl<R: Read, W: Write> Interpreter<R, W> {
    fn compress_codes(codes: &Vec<u8>) -> Result<Vec<BfInstr>, String> {
        let mut instructions = Vec::<BfInstr>::with_capacity(codes.len());

        for &b in codes.iter() {
            if let Some(instr) = BfInstr::from_byte(b) {
                if instructions.is_empty()
                    || matches!(
                        instr,
                        BfInstr::WriteByte
                            | BfInstr::ReadByte
                            | BfInstr::JmpFwd(_)
                            | BfInstr::JmpBwd(_)
                    )
                    || std::mem::discriminant(instructions.last().unwrap())
                        != std::mem::discriminant(&instr)
                {
                    instructions.push(instr);
                    continue;
                }

                match instr {
                    BfInstr::MoveDataPtr(offset) => {
                        if let BfInstr::MoveDataPtr(cur_offset) = instructions.last_mut().unwrap() {
                            *cur_offset += offset;
                        } else {
                            unreachable!(); // The `if` condition above guarantees this won't happen
                        }
                    }
                    BfInstr::WrappingAddCurByte(diff) => {
                        if let BfInstr::WrappingAddCurByte(cur_diff) =
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

        Ok(instructions)
    }

    fn idiom_recognize(instrs: &Vec<BfInstr>) -> Vec<ExtInstr> {
        let mut ext_instrs = Vec::<ExtInstr>::with_capacity(instrs.len());

        let mut i = 0;
        while i < instrs.len() {
            if i + 2 < instrs.len() {
                // Recognize "[-]" and "[+]" as ClearByte
                if let (
                    &BfInstr::JmpFwd(fwd_pos),
                    &BfInstr::WrappingAddCurByte(diff),
                    &BfInstr::JmpBwd(bwd_pos),
                ) = (&instrs[i], &instrs[i + 1], &instrs[i + 2])
                {
                    debug_assert_eq!(fwd_pos, BfInstr::JMP_POS_PLACEHOLDER);
                    debug_assert_eq!(bwd_pos, BfInstr::JMP_POS_PLACEHOLDER);

                    if diff == 1 || diff == u8::MAX {
                        ext_instrs.push(ExtInstr::ClearByte);
                        i += 3;
                        continue;
                    }
                }

                // Recognize "[>]" and "[<]" as FindZeroByte
                if let (
                    &BfInstr::JmpFwd(fwd_pos),
                    &BfInstr::MoveDataPtr(offset),
                    &BfInstr::JmpBwd(bwd_pos),
                ) = (&instrs[i], &instrs[i + 1], &instrs[i + 2])
                {
                    debug_assert_eq!(fwd_pos, BfInstr::JMP_POS_PLACEHOLDER);
                    debug_assert_eq!(bwd_pos, BfInstr::JMP_POS_PLACEHOLDER);

                    ext_instrs.push(ExtInstr::FindZeroByte(offset));
                    i += 3;
                    continue;
                }
            }

            ext_instrs.push(ExtInstr::from_bf_instr(&instrs[i]));
            i += 1;
        }

        ext_instrs
    }

    fn process_jmps(&mut self) -> Result<(), String> {
        let mut stk = Vec::<usize>::with_capacity(self.instructions.len());
        let len = self.instructions.len();
        for i in 0..len {
            match self.instructions[i] {
                ExtInstr::JmpFwd(pos) => {
                    debug_assert_eq!(pos, ExtInstr::JMP_POS_PLACEHOLDER);

                    stk.push(i);
                }
                ExtInstr::JmpBwd(pos) => {
                    debug_assert_eq!(pos, ExtInstr::JMP_POS_PLACEHOLDER);

                    if let Some(open_pos) = stk.pop() {
                        let close_pos = i;
                        if let ExtInstr::JmpFwd(ref mut open_match_pos) =
                            self.instructions[open_pos]
                        {
                            *open_match_pos = close_pos;
                        } else {
                            unreachable!();
                        }
                        if let ExtInstr::JmpBwd(ref mut close_match_pos) =
                            self.instructions[close_pos]
                        {
                            *close_match_pos = open_pos;
                        } else {
                            unreachable!();
                        }
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
                stk.pop().unwrap()
            ));
        }

        Ok(())
    }

    pub fn try_new(executor: Executor<R, W>, codes: Vec<u8>) -> Result<Self, String> {
        let mut interpreter = Interpreter {
            executor: executor,
            instructions: Self::idiom_recognize(&Self::compress_codes(&codes)?),
            instr_ptr: 0,
        };
        interpreter.process_jmps()?;
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
            ExtInstr::MoveDataPtr(offset) => self.executor.move_data_ptr(offset)?,
            ExtInstr::WrappingAddCurByte(diff) => self.executor.add_cur_byte(diff),
            ExtInstr::WriteByte => self.executor.write_byte()?,
            ExtInstr::ReadByte => self.executor.read_byte()?,
            ExtInstr::JmpFwd(pos) => {
                if self.executor.cur_byte() == 0 {
                    self.instr_ptr = pos;
                }
            }
            ExtInstr::JmpBwd(pos) => {
                if self.executor.cur_byte() != 0 {
                    self.instr_ptr = pos;
                }
            }
            ExtInstr::ClearByte => self.executor.clear_byte(),
            ExtInstr::FindZeroByte(step) => self.executor.find_zero_byte(step)?,
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
