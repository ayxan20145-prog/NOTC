mod ast;
mod cli;
mod codegen;
mod lexer;
mod parser;
mod token;

use clap::Parser as ClapParser;
use cli::Cli;
use codegen::compile;
use lexer::Lexer;
use parser::Parser;
use std::{fs, process::Command};

fn main() {
    let args = Cli::parse();

    let content = fs::read_to_string(&args.input).expect("Failed to read program");

    let mut lexer = Lexer::new(&content);

    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);

    let program = parser.parse_program();

    let assembly = compile(&program);

    if args.assemble {
        fs::write(&format!("{}.asm", args.output), assembly).unwrap();
        return;
    }

    fs::write("output.tmp.asm", assembly).unwrap();

    Command::new("nasm")
        .args(["-f", "elf64", "output.tmp.asm", "-o", "output.tmp.o"])
        .status()
        .unwrap();

    if args.compile {
        fs::remove_file("output.tmp.asm").unwrap();
        return;
    }

    Command::new("ld")
        .args(["output.tmp.o", "-o", &args.output])
        .status()
        .unwrap();

    fs::remove_file("output.tmp.asm").unwrap();
    fs::remove_file("output.tmp.o").unwrap();
}
