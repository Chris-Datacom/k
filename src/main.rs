//! Command-line entry point for the K compiler tools.

use std::env;
use std::fs;
use std::process::ExitCode;
use std::str::FromStr;

use k_compiler::driver::{
    combine_programs, compile_sources, compile_sources_for_target,
};
use k_compiler::lexer::{Lexer, SpannedToken, TokenKind};
use k_compiler::parser::parse;
use k_compiler::sema::check;
use k_compiler::target::Target;

fn main() -> ExitCode {
    let mut arguments = env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = arguments.first().cloned() else {
        print_usage();
        return ExitCode::from(2);
    };
    arguments.remove(0);

    match command.as_str() {
        "lex" => {
            let Some(path) = arguments.first() else {
                eprintln!("k lex: expected a .k source file");
                return ExitCode::from(2);
            };
            lex_file(path)
        }
        "parse" => {
            if arguments.is_empty() {
                eprintln!("k parse: expected at least one .k source file");
                return ExitCode::from(2);
            }
            parse_files(&arguments)
        }
        "check" => {
            if arguments.is_empty() {
                eprintln!("k check: expected at least one .k source file");
                return ExitCode::from(2);
            }
            check_files(&arguments)
        }
        "emit" => {
            if arguments.is_empty() {
                eprintln!("k emit: expected at least one .k source file");
                return ExitCode::from(2);
            }
            emit_files(&arguments)
        }
        "compile" => {
            if arguments.is_empty() {
                eprintln!("k compile: expected input .k source file(s) and an output file");
                return ExitCode::from(2);
            }
            let mut inputs = Vec::new();
            let mut output = None;
            let mut target = None;

            let mut i = 0;
            while i < arguments.len() {
                if arguments[i] == "-o" {
                    if i + 1 < arguments.len() {
                        output = Some(arguments[i + 1].clone());
                        i += 2;
                        continue;
                    } else {
                        eprintln!("k compile: -o requires an output file argument");
                        return ExitCode::from(2);
                    }
                }
                if let Ok(t) = Target::from_str(&arguments[i]) {
                    target = Some(t);
                    i += 1;
                    continue;
                }
                inputs.push(arguments[i].clone());
                i += 1;
            }

            if output.is_none() {
                if inputs.len() >= 2 {
                    output = inputs.pop();
                } else {
                    eprintln!("k compile: expected output assembly file");
                    return ExitCode::from(2);
                }
            }

            let output = output.unwrap();
            let target = target.unwrap_or_default();
            compile_files(&inputs, &output, target)
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

fn parse_files(paths: &[String]) -> ExitCode {
    let mut programs = Vec::new();
    for path in paths {
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("k parse: cannot read `{path}`: {error}");
                return ExitCode::from(1);
            }
        };
        match parse(&source) {
            Ok(program) => programs.push(program),
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::from(1);
            }
        }
    }
    match combine_programs(&programs) {
        Ok(combined) => {
            println!("{combined:#?}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn check_files(paths: &[String]) -> ExitCode {
    let mut programs = Vec::new();
    for path in paths {
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("k check: cannot read `{path}`: {error}");
                return ExitCode::from(1);
            }
        };
        match parse(&source) {
            Ok(program) => programs.push(program),
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::from(1);
            }
        }
    }
    let combined = match combine_programs(&programs) {
        Ok(combined) => combined,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::from(1);
        }
    };
    match check(&combined) {
        Ok(()) => {
            for path in paths {
                println!("ok: {path}");
            }
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

fn emit_files(paths: &[String]) -> ExitCode {
    let mut sources = Vec::new();
    for path in paths {
        let source = match read_source(path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("k emit: cannot read `{path}`: {error}");
                return ExitCode::from(1);
            }
        };
        sources.push(source);
    }
    let source_refs = sources.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    match compile_sources(&source_refs) {
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

fn compile_files(inputs: &[String], output: &str, target: Target) -> ExitCode {
    let mut sources = Vec::new();
    for input in inputs {
        let source = match read_source(input) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("k compile: cannot read `{input}`: {error}");
                return ExitCode::from(1);
            }
        };
        sources.push(source);
    }
    let source_refs = sources.iter().map(|s| s.as_str()).collect::<Vec<_>>();
    let assembly = match compile_sources_for_target(&source_refs, target) {
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
    println!("K compiler prototype\n\nUsage:\n  k lex <file.k>                         Print the source tokens\n  k parse <file.k...>                    Print the parsed AST\n  k check <file.k...>                    Check names and types\n  k emit <file.k...>                     Emit default-target assembly\n  k compile <in.k...> <out.s> [target]   Compile for a target\n  k compile -o <out.s> <in.k...> [target] Compile for a target\n  k help                                 Show this help\n\nTargets:\n  x86_64-unknown-linux-gnu (default)\n  x86_64-krumpyos\n  aarch64-krumpyos (not implemented)\n");
}
