mod bf;

use brain_fucker::run_interpreter;
use clap::Parser;
use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::bf::{executor::Executor, interpreter::Interpreter};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    filename: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let file = File::open(&args.filename)?;
    let reader = BufReader::new(file);
    let code = reader
        .bytes()
        .filter(|b_res| {
            if let Ok(b) = b_res {
                !b.is_ascii_whitespace()
            } else {
                return false;
            }
        })
        .collect::<Result<Vec<u8>, std::io::Error>>()?;

    run_interpreter(code, std::io::stdin(), std::io::stdout())?;

    Ok(())
}
