//! Host-independent compiler orchestration.

use std::fmt;

use crate::{codegen, parser, sema};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompileError {
    Parse(parser::ParseError),
    Semantic(Vec<sema::SemanticError>),
    Codegen(codegen::CodegenError),
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(error) => error.fmt(formatter),
            Self::Semantic(errors) => {
                for (index, error) in errors.iter().enumerate() {
                    if index != 0 {
                        writeln!(formatter)?;
                    }
                    error.fmt(formatter)?;
                }
                Ok(())
            }
            Self::Codegen(error) => error.fmt(formatter),
        }
    }
}

/// Compile one complete K source buffer into deterministic target assembly.
pub fn compile_source(source: &str) -> Result<String, CompileError> {
    let program = parser::parse(source).map_err(CompileError::Parse)?;
    sema::check(&program).map_err(CompileError::Semantic)?;
    codegen::emit(&program).map_err(CompileError::Codegen)
}

#[cfg(test)]
mod tests {
    use super::{compile_source, CompileError};

    #[test]
    fn compiles_source_without_filesystem_access() {
        let assembly = compile_source("int main() { return 42; }").unwrap();
        assert!(assembly.contains(".globl main"));
    }

    #[test]
    fn returns_structured_diagnostics() {
        let error = compile_source("int main() { return missing; }").unwrap_err();
        assert!(matches!(error, CompileError::Semantic(_)));
        assert!(error.to_string().contains("undefined name"));
    }

    #[test]
    fn compiles_the_k_bootstrap_lexer() {
        let source = include_str!("../compiler/lexer.k");
        let assembly = compile_source(source).expect("bootstrap lexer should compile");
        assert!(assembly.contains(".globl lex_one"));
        assert!(assembly.contains("add rax, 8"));
        assert!(assembly.contains("mov QWORD PTR [rdi], rax"));
    }
}
