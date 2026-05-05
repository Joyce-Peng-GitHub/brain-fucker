use std::{
    collections::HashMap,
    io::{BufReader, Read, Write},
};

pub struct Executor<R, W> {
    data: Vec<u8>,
    data_ptr: usize,
    reader: BufReader<R>,
    writer: W,
}

impl<R: Read, W: Write> Executor<R, W> {
    pub const DEFAULT_CAPACITY: usize = 64 * (1 << 10) * (1 << 10); // 64 MiB

    pub fn with_capacity(input: R, writer: W, cap: usize) -> Self {
        Self {
            data: vec![0; cap],
            data_ptr: 0,
            reader: BufReader::new(input),
            writer,
        }
    }

    pub fn new(input: R, writer: W) -> Self {
        Self::with_capacity(input, writer, Self::DEFAULT_CAPACITY.max(1))
    }

    pub fn move_data_ptr(&mut self, offset: isize) -> Result<(), String> {
        if self.data_ptr as isize + offset < 0 {
            return Err(format!(
                "Pointer underflow: cannot move pointer to negative index (current: {}, offset: {})",
                self.data_ptr, offset
            ));
        }
        self.data_ptr = (self.data_ptr as isize + offset) as usize;
        if self.data_ptr >= self.data.len() {
            self.data.resize(self.data_ptr + 1, 0);
        }
        Ok(())
    }

    pub fn wrapping_add_cur_byte(&mut self, diff: u8) {
        self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_add(diff);
    }

    pub fn read_byte(&mut self) -> std::io::Result<()> {
        let n = self
            .reader
            .read(&mut self.data[self.data_ptr..self.data_ptr + 1])?;
        if n == 0 {
            self.data[self.data_ptr] = u8::MAX; // EOF
        }
        Ok(())
    }

    pub fn cur_byte(&self) -> u8 {
        self.data[self.data_ptr]
    }

    pub fn write_byte(&mut self) -> std::io::Result<()> {
        self.writer
            .write_all(&self.data[self.data_ptr..self.data_ptr + 1])?;
        // self.writer.flush()?;
        Ok(())
    }

    pub fn set_byte(&mut self, val: u8) {
        self.data[self.data_ptr] = val;
    }

    pub fn find_zero_byte(&mut self, step: isize) -> Result<(), String> {
        if step == 0 {
            return Err("Step cannot be zero".to_string());
        }

        while self.cur_byte() != 0 {
            self.move_data_ptr(step)?;
            if self.data_ptr >= self.data.len() {
                self.data.resize(self.data_ptr + 1, 0);
                break;
            }
        }

        Ok(())
    }

    pub fn batch_wrapping_add_mul(
        &mut self,
        offset_diffs: &HashMap<isize, u8>,
        is_minus: bool,
    ) -> Result<(), String> {
        let mul = if is_minus {
            self.cur_byte()
        } else {
            self.cur_byte().wrapping_neg()
        };
        if mul == 0 {
            return Ok(());
        }
        self.set_byte(0);
        for (offset, diff) in offset_diffs {
            if self.data_ptr as isize + offset < 0 {
                return Err(format!(
                    "Pointer underflow: cannot move pointer to negative index (current: {}, offset: {})",
                    self.data_ptr, offset
                ));
            }
            let target_ptr = (self.data_ptr as isize + offset) as usize;
            if target_ptr >= self.data.len() {
                self.data.resize(target_ptr + 1, 0);
            }
            self.data[target_ptr] = self.data[target_ptr].wrapping_add(diff.wrapping_mul(mul));
        }
        Ok(())
    }
}
