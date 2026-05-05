use std::{
    collections::HashMap,
    io::{Read, Write},
    usize,
};

use crate::bf::executor::Executor;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Instr {
    MoveDataPtr(isize),
    WrappingAddCurByte(u8),
    WriteByte,
    ReadByte,
    JmpFwd(usize),
    JmpBwd(usize),
    SetByte(u8),
    FindZeroByte(isize),                          // step
    BatchWrappingAddMul((HashMap<isize, u8>, bool)), // offset -> diff, is_minus
}

impl Instr {
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

pub struct Interpreter<R, W> {
    executor: Executor<R, W>,
    instrs: Vec<Instr>,
    instr_ptr: usize,
}

impl<R: Read, W: Write> Interpreter<R, W> {
    fn replace_set_byte_idiom(instrs: &mut Vec<Instr>) -> bool {
        if instrs.len() < 2 {
            return false;
        }
        if let (&Instr::SetByte(val), &Instr::WrappingAddCurByte(diff)) =
            (&instrs[instrs.len() - 2], &instrs[instrs.len() - 1])
        {
            instrs.pop();
            *instrs.last_mut().unwrap() = Instr::SetByte(u8::wrapping_add(val, diff));
            return true;
        }

        if instrs.len() < 3 {
            return false;
        }
        if matches!(
            (
                &instrs[instrs.len() - 3],
                &instrs[instrs.len() - 2],
                &instrs[instrs.len() - 1]
            ),
            (
                Instr::JmpFwd(_),
                Instr::WrappingAddCurByte(1) | Instr::WrappingAddCurByte(u8::MAX),
                Instr::JmpBwd(_)
            )
        ) {
            instrs.pop();
            instrs.pop();
            *instrs.last_mut().unwrap() = Instr::SetByte(0);
            return true;
        }
        return false;
    }
    fn replace_find_zero_byte_idiom(instrs: &mut Vec<Instr>) -> bool {
        if instrs.len() < 3 {
            return false;
        }

        if matches!(
            (
                &instrs[instrs.len() - 3],
                &instrs[instrs.len() - 2],
                &instrs[instrs.len() - 1]
            ),
            (Instr::JmpFwd(_), Instr::MoveDataPtr(_), Instr::JmpBwd(_))
        ) {
            instrs.pop();

            let offset = if let Instr::MoveDataPtr(offset) = instrs.pop().unwrap() {
                offset
            } else {
                unreachable!();
            };
            debug_assert_ne!(offset, 0);

            *instrs.last_mut().unwrap() = Instr::FindZeroByte(offset);
            return true;
        }

        return false;
    }
    fn replace_batch_wrapping_add_mul_idiom(instrs: &mut Vec<Instr>) -> bool {
        if instrs.len() < 6 || !matches!(instrs.last(), Some(Instr::JmpBwd(_))) {
            return false;
        }

        let mut beg = instrs.len() - 2;
        loop {
            match instrs[beg] {
                Instr::MoveDataPtr(_) | Instr::WrappingAddCurByte(_) => {}
                Instr::JmpFwd(_) => break,
                _ => return false,
            }
            if beg == 0 {
                return false;
            }
            beg -= 1;
        }

        let mut offset_diffs = HashMap::<isize, u8>::new();
        let mut cur_offset = 0isize;
        for instr in &instrs[beg + 1..instrs.len() - 1] {
            match instr {
                Instr::MoveDataPtr(offset) => cur_offset += offset,
                Instr::WrappingAddCurByte(diff) => {
                    let cur_diff = offset_diffs.entry(cur_offset).or_insert(0);
                    *cur_diff = cur_diff.wrapping_add(*diff);
                }
                _ => unreachable!(),
            }
        }

        if cur_offset != 0 {
            return false;
        }

        let base_diff = offset_diffs.remove(&0).unwrap_or(0);
        let is_minus = base_diff == u8::MAX;
        if !is_minus && base_diff != 1 {
            return false;
        }

        instrs.truncate(beg);
        instrs.push(Instr::BatchWrappingAddMul((offset_diffs, is_minus)));
        return true;
    }

    fn replace_idioms(instrs: &mut Vec<Instr>) -> bool {
        return Self::replace_set_byte_idiom(instrs)
            || Self::replace_find_zero_byte_idiom(instrs)
            || Self::replace_batch_wrapping_add_mul_idiom(instrs);
    }

    fn parse_codes(codes: &Vec<u8>) -> Result<Vec<Instr>, String> {
        let mut instrs = Vec::<Instr>::with_capacity(codes.len());

        for &b in codes.iter() {
            if let Some(instr) = Instr::from_byte(b) {
                if instrs.is_empty()
                    || matches!(
                        instr,
                        Instr::ReadByte | Instr::WriteByte | Instr::JmpFwd(_) | Instr::JmpBwd(_)
                    )
                    || std::mem::discriminant(instrs.last().unwrap())
                        != std::mem::discriminant(&instr)
                {
                    while Self::replace_idioms(&mut instrs) {}
                    instrs.push(instr);
                    continue;
                }

                match instr {
                    Instr::MoveDataPtr(offset) => {
                        if let Instr::MoveDataPtr(cur_offset) = instrs.last_mut().unwrap() {
                            *cur_offset += offset;
                        } else {
                            unreachable!(); // The `if` condition above guarantees this won't happen
                        }
                    }
                    Instr::WrappingAddCurByte(diff) => {
                        if let Instr::WrappingAddCurByte(cur_diff) = instrs.last_mut().unwrap() {
                            *cur_diff = cur_diff.wrapping_add(diff);
                        } else {
                            unreachable!(); // The `if` condition above guarantees this won't happen
                        }
                    }
                    _ => unreachable!(),
                }
            }
        }

        Ok(instrs)
    }

    fn process_jmps(&mut self) -> Result<(), String> {
        let mut stk = Vec::<usize>::with_capacity(self.instrs.len());
        let len = self.instrs.len();
        for i in 0..len {
            match self.instrs[i] {
                Instr::JmpFwd(pos) => {
                    debug_assert_eq!(pos, Instr::JMP_POS_PLACEHOLDER);

                    stk.push(i);
                }
                Instr::JmpBwd(pos) => {
                    debug_assert_eq!(pos, Instr::JMP_POS_PLACEHOLDER);

                    if let Some(open_pos) = stk.pop() {
                        let close_pos = i;
                        if let Instr::JmpFwd(ref mut open_match_pos) = self.instrs[open_pos] {
                            *open_match_pos = close_pos;
                        } else {
                            unreachable!();
                        }
                        if let Instr::JmpBwd(ref mut close_match_pos) = self.instrs[close_pos] {
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
            instrs: Self::parse_codes(&codes)?,
            instr_ptr: 0,
        };
        interpreter.process_jmps()?;
        Ok(interpreter)
    }

    pub fn exec_once(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.instr_ptr >= self.instrs.len() {
            return Err(format!(
                "Instruction pointer out of bounds: {} >= {}",
                self.instr_ptr,
                self.instrs.len()
            )
            .into());
        }

        match &self.instrs[self.instr_ptr] {
            &Instr::MoveDataPtr(offset) => self.executor.move_data_ptr(offset)?,
            &Instr::WrappingAddCurByte(diff) => self.executor.wrapping_add_cur_byte(diff),
            &Instr::WriteByte => self.executor.write_byte()?,
            &Instr::ReadByte => self.executor.read_byte()?,
            &Instr::JmpFwd(pos) => {
                if self.executor.cur_byte() == 0 {
                    self.instr_ptr = pos;
                }
            }
            &Instr::JmpBwd(pos) => {
                if self.executor.cur_byte() != 0 {
                    self.instr_ptr = pos;
                }
            }
            &Instr::SetByte(val) => self.executor.set_byte(val),
            &Instr::FindZeroByte(step) => self.executor.find_zero_byte(step)?,
            &Instr::BatchWrappingAddMul((ref offset_diffs, is_minus)) => {
                self.executor.batch_wrapping_add_mul(offset_diffs, is_minus)?
            }
        }

        self.instr_ptr += 1;
        Ok(())
    }

    pub fn exec(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.instr_ptr < self.instrs.len() {
            self.exec_once()?;
        }
        Ok(())
    }
}
