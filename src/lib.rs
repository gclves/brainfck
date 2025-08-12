mod parser;
mod tokenizer;
mod vm;

use std::io::{Read, Write, stdin, stdout};

use parser::parse;
use tokenizer::tokenize;
use vm::VM;

use crate::vm::compile;

pub fn repl() {
    let mut machine = VM::new();
    let mut line = String::new();

    loop {
        prompt();

        if let Some(input) = read(&mut line) {
            let tokens = tokenize(&input);
            let parse_result = parse(&tokens);

            match parse_result {
                Ok(parsed) => {
                    let bytecode = compile(&parsed);
                    machine.eval(&bytecode)
                }
                Err(error) => {
                    println!("Parse error: {}", error);
                    continue;
                }
            }
        } else {
            println!();
            break;
        }
    }
}

fn read(line: &mut String) -> Option<String> {
    line.clear();
    match stdin().read_to_string(line) {
        Err(e) => {
            dbg!(e);
            None
        }
        Ok(0) => None,
        Ok(_) => Some(line.to_string()),
    }
}

fn prompt() {
    print!("CCBF> ");
    stdout().flush().unwrap();
}
