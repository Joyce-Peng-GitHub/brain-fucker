<div align="center">

# Brain Fucker

Language: [English](README.md), [简体中文](README.zh-CN.md).

</div>

## Prerequisites

The build process requires the [Rust toolchain](https://rust-lang.org/tools/install/).

## Build and Run

Execute the following in the project root directory:
```bash
cargo build
```
You can add the `--release` flag to build the project in release mode.

Usage
```bash
$ ./bf-interpreter --help
Usage: bf-interpreter <FILENAME>

Arguments:
  <FILENAME>  

Options:
  -h, --help     Print help
  -V, --version  Print version
```
For example, you can run:
```bash
./bf-interpreter code.bf
```
to interpret and execute the `code.bf` code. The interpreter **uses its own standard input and standard output as the standard input and standard output for the BrainFuck code**, respectively.

You can run the following (adding `--release` provides better performance):
```bash
cargo test
```
to test the 7 sets of samples provided in the contest.

## Features

1. Parses command-line arguments. You can use `bf-interpreter --help` to view usage instructions.
2. Reads BrainFuck code from a file (ignoring invalid BrainFuck characters) and stores it using the `Instr` enum type.
3. Instruction folding optimization: Folds adjacent instructions of the same type.
4. Idiom recognition optimization: Recognizes common loop patterns and replaces them with extended instructions.
5. Jump location pre-computation: Pre-computes the jump destination for every `[` and `]` instruction.
6. Interpretation and execution.

## AI Usage Declaration

- Used Copilot for short snippet completions. **All** code was written through a combination of manual typing and Copilot completion.
- Used the web version of Gemini 3.1 Pro to brainstorm optimization ideas, find bugs and translate this document.