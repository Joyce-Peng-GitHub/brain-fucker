use std::io::{BufReader, Read, Write};

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
        Self::with_capacity(input, writer, Self::DEFAULT_CAPACITY)
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

    pub fn add_cur_byte(&mut self, diff: u8) {
        self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_add(diff);
    }

    pub fn sub_cur_byte(&mut self, diff: u8) {
        self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_sub(diff);
    }

    pub fn read_byte(&mut self) -> std::io::Result<()> {
        self.reader
            .read_exact(&mut self.data[self.data_ptr..(self.data_ptr + 1)])?;
        Ok(())
    }

    pub fn cur_byte(&self) -> u8 {
        self.data[self.data_ptr]
    }

    pub fn print_byte(&mut self) -> std::io::Result<()> {
        self.writer
            .write_all(&self.data[self.data_ptr..self.data_ptr + 1])?;
        // self.writer.flush()?;
        Ok(())
    }
}
