mod bf;

use std::io::{Read, Write};

use crate::bf::{executor::Executor, interpreter::Interpreter};

pub fn run_interpreter(
    code: Vec<u8>,
    input: impl Read,
    mut output: impl Write,
) -> Result<(), Box<dyn std::error::Error>> {
    output.write_all(b"")?; // Ensure output is flushed before execution

    let executor = Executor::new(input, &mut output);
    let mut interpreter = Interpreter::try_new(executor, code)?;

    interpreter.exec()?;

    Ok(())
}
