//! Command-line entry point for the K compiler tools.

use std::env;
use std::fs;
use std::process::ExitCode;
use std::str::FromStr;

use k_compiler::driver::{compile_source, compile_source_for_target};
use k_compiler::lexer::{Lexer, SpannedToken, TokenKind};
use k_compiler::parser::parse;
use k_compiler::sema::check;
use k_compiler::target::Target;

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
        "parse" => {
            let Some(path) = arguments.next() else {
                eprintln!("k parse: expected a .k source file");
                return ExitCode::from(2);
            };
            parse_file(&path)
        }
        "check" => {
            let Some(path) = arguments.next() else {
                eprintln!("k check: expected a .k source file");
                return ExitCode::from(2);
            };
            check_file(&path)
        }
        "emit" => {
            let Some(path) = arguments.next() else {
                eprintln!("k emit: expected a .k source file");
                return ExitCode::from(2);
            };
            emit_file(&path)
        }
        "compile" => {
            let Some(input) = arguments.next() else {
                eprintln!("k compile: expected an input .k source file");
                return ExitCode::from(2);
            };
            let Some(output) = arguments.next() else {
                eprintln!("k compile: expected an output assembly file");
                return ExitCode::from(2);
            };
            let target = arguments
                .next()
                .map(|value| Target::from_str(&value))
                .transpose();
            match target {
                Ok(Some(target)) => compile_file(&input, &output, target),
                Ok(None) => compile_file(&input, &output, Target::default()),
                Err(error) => {
                    eprintln!("k compile: {error}");
                    ExitCode::from(2)
                }
            }
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

fn parse_file(path: &str) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("k parse: cannot read `{path}`: {error}");
            return ExitCode::from(1);
        }
    };

    match parse(&source) {
        Ok(program) => {
            println!("{program:#?}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn check_file(path: &str) -> ExitCode {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("k check: cannot read `{path}`: {error}");
            return ExitCode::from(1);
        }
    };

    let program = match parse(&source) {
        Ok(program) => program,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };

    match check(&program) {
        Ok(()) => {
            println!("ok: {path}");
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in errors {
                eprintln!("{error}");
            }
            ExitCode::from(1)
        }
    }
}

fn emit_file(path: &str) -> ExitCode {
    let source = match read_source(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("k emit: cannot read `{path}`: {error}");
            return ExitCode::from(1);
        }
    };
    match compile_source(&source) {
        Ok(assembly) => {
            print!("{assembly}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn compile_file(input: &str, output: &str, target: Target) -> ExitCode {
    let source = match read_source(input) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("k compile: cannot read `{input}`: {error}");
            return ExitCode::from(1);
        }
    };
    let assembly = match compile_source_for_target(&source, target) {
        Ok(assembly) => assembly,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };
    match fs::write(output, assembly) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("k compile: cannot write `{output}`: {error}");
            ExitCode::from(1)
        }
    }
}

fn read_source(path: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    String::from_utf8(bytes).map_err(|error| format!("source is not UTF-8: {error}"))
}

fn print_usage() {
    println!("K compiler prototype\n\nUsage:\n  k lex <file.k>                         Print the source tokens\n  k parse <file.k>                       Print the parsed AST\n  k check <file.k>                       Check names and types\n  k emit <file.k>                        Emit default-target assembly\n  k compile <in.k> <out.s> [target]      Compile for a target\n  k help                                 Show this help\n\nTargets:\n  x86_64-unknown-linux-gnu (default)\n  x86_64-krumpyos\n  aarch64-krumpyos (not implemented)\n");
}
