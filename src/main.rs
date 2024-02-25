#![allow(unused)]

use std::env;
use std::process;
use rustyline::error::ReadlineError;

mod icps;
mod token;
mod scanner;
mod ast;
mod parser;
mod environment;
mod interpreter;

fn main() -> Result<(), ReadlineError> {
    let mut interpreter = interpreter::Interpreter::new();
    let args: Vec<String> = env::args().collect();

    let non_option_args = args.iter().skip(1).filter(|arg| !arg.starts_with('-')).count();

    if non_option_args > 1 {
        println!("Usage: icps <file> [OPTIONS] or icps [OPTIONS] for REPL.");
        process::exit(64);
    }

    if args.len() > 1 && !args[1].starts_with('-') {
        icps::run_file(&args[1], &mut interpreter);
    } else {
        icps::run_prompt(&mut interpreter)?;
    }

    Ok(())
}

