#![allow(unused)]

use std::env;
use std::process;
use rustyline::error::ReadlineError;

mod icps;
mod token;
mod scanner;
mod ast;
mod parser;
mod printer;
mod environment;

fn main() -> Result<(), ReadlineError> {
    let args: Vec<String> = env::args().collect();

    if args.iter().skip(1).any(|arg| !arg.starts_with('-')) {
        println!("Usage: icps <file> [OPTIONS] or icps [OPTIONS] for REPL.",);
        process::exit(64);
    }

    if args.len() > 1 && !args[1].starts_with('-') {
        icps::run_file(&args[1]);
    } else {
        icps::run_prompt()?;
    }

    Ok(())
}
