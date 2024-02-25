use std::fmt::{Debug, format, Formatter};
use std::{env, fs};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process;
use lazy_static::lazy_static;
use crate::{interpreter, parser, scanner};
use crate::scanner::Loc;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use rustyline::history::FileHistory;
use crate::ast::{Expr, Stmt};
use crate::token::Value;
use chrono::Local;
use crate::environment::Environment;
use crate::interpreter::Interpreter;

pub fn run_file(path: &str, interpreter: &mut Interpreter) {
    match fs::read_to_string(Path::new(path)) {
        Ok(contents) => {
            if let Err(e) = run(&contents, interpreter) {
                eprintln!("{}", e);
                process::exit(70);
            }
        },
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            process::exit(65);
        },
    };
}


pub fn run_prompt(interpreter: &mut Interpreter) -> Result<(), ReadlineError> {
    let mut env = Environment::new();
    let mut rl = Editor::<(), FileHistory>::new()?;
    rl.load_history("history.txt");
    println!("ICPS 0.1.0 ({}) [Rust ICPS Interpreter 1.75.0 Nightly]", Local::now().format("2024-02-16"));
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).expect("TODO: panic message");
                if let Err(e) = run(&format!("{}\n", line), interpreter) {
                    eprintln!("{}", e);
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("KeyboardInterrupt");
                continue;
            },
            Err(ReadlineError::Eof) => {
                println!("EOF marker received, terminating");
                break;
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };
    }
    rl.save_history("history.txt").unwrap(); // Properly handle this in production code

    Ok(())
}

pub fn run(source: &str, interpreter: &mut Interpreter) -> Result<(), Error> {
    let mut scanner = scanner::Scanner::new(source);
    let scanned = scanner.scan();
    match scanned {
        Ok(tokens) => {
            let mut parser = parser::Parser::new(&tokens);
            match parser.parse() {
                Ok(tree) => {
                    return interpreter.interpret(tree);
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

pub fn warn(line: usize, col: usize, message: &str) {
    eprintln!("[line {}, column {}] Warning: {}", line, col, message);
}

pub fn panic(message: &str) {
    eprintln!("Interpreter panic! This is probably not your fault, but instead an internal bug due to how new and poorly-tested the interpreter is. Please report this by opening an issue on https://github.com/eagely/icps: {}", message);
    process::exit(70);
}

#[derive(Debug)]
pub struct Error {
    loc: Loc,
    message: String,
}

impl Error {
    pub fn new(loc: Loc, message: &str) -> Self {
        Self {
            loc,
            message: message.to_owned(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "[{}:{}] {}", self.loc.line, self.loc.col, self.message)
    }
}

impl std::error::Error for Error {}