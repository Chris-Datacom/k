//! Command-line entry point for the K compiler tools.

use std::env;
use std::fs;
use std::process::ExitCode;

use k_compiler::lexer::{Lexer, SpannedToken, TokenKind};

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1);
    let Some(command) = arguments.next() else {
        print_usage();
        return ExitCode::from(2);
    };

    match command.as_str() {
        "lex" => {
            let Some(path) = arguments.next() else {
                eprintln!("k lex: expected a .k source file");
                return ExitCode::from(2);
            };
            lex_file(&path)
        }
        "help" | "--help" | "-h" => {
            print_usage();
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("k: unknown command `{command}`");
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn lex_file(path: &str) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("k lex: cannot read `{path}`: {error}");
            return ExitCode::from(1);
        }
    };

    let mut had_error = false;
    for item in Lexer::new(&source) {
        match item {
            Ok(SpannedToken { token, span }) => println!("{span}: {token}"),
            Err(error) => {
                eprintln!("{error}");
                had_error = true;
            }
        }
    }

    if !had_error {
        println!("{}", TokenKind::Eof);
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn print_usage() {
    println!("K compiler prototype\n\nUsage:\n  k lex <file.k>    Print the source tokens\n  k help            Show this help\n");
}
