//! Host-independent compiler orchestration.

use std::fmt;

use crate::{codegen, parser, sema, target::Target};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CompileError {
    Parse(parser::ParseError),
    Semantic(Vec<sema::SemanticError>),
    Codegen(codegen::CodegenError),
    UnsupportedTarget(Target),
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
            Self::UnsupportedTarget(target) => {
                write!(formatter, "target `{}` is not implemented", target.name())
            }
        }
    }
}

/// Compile one complete K source buffer into deterministic target assembly.
pub fn compile_source(source: &str) -> Result<String, CompileError> {
    compile_source_for_target(source, Target::default())
}

/// Compile one complete K source buffer for an explicit target.
pub fn compile_source_for_target(
    source: &str,
    target: Target,
) -> Result<String, CompileError> {
    if !target.is_implemented() {
        return Err(CompileError::UnsupportedTarget(target));
    }
    let program = parser::parse(source).map_err(CompileError::Parse)?;
    sema::check(&program).map_err(CompileError::Semantic)?;
    codegen::emit_for_target(&program, target).map_err(CompileError::Codegen)
}

#[cfg(test)]
mod tests {
    use super::{compile_source, compile_source_for_target, CompileError};
    use crate::target::Target;

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
    fn rejects_unimplemented_targets_before_codegen() {
        let error = compile_source_for_target(
            "int main() { return 0; }",
            Target::Aarch64KrumpyOs,
        )
        .unwrap_err();
        assert!(matches!(error, CompileError::UnsupportedTarget(Target::Aarch64KrumpyOs)));
    }

    #[test]
    fn compiles_the_k_bootstrap_lexer() {
        let source = include_str!("../compiler/lexer.k");
        let assembly = compile_source(source).expect("bootstrap lexer should compile");
        assert!(assembly.contains(".globl lex_one"));
        assert!(assembly.contains("add rax, 8"));
        assert!(assembly.contains("mov QWORD PTR [rdi], rax"));
        assert!(assembly.contains(".globl next_token"));
        assert!(assembly.contains(".globl lexer_init"));
    }

    #[test]
    fn compiles_the_k_bootstrap_parser() {
        let source = include_str!("../compiler/parser.k");
        let assembly = compile_source(source).expect("bootstrap parser should compile");
        assert!(assembly.contains(".globl parse_program"));
        assert!(assembly.contains(".globl parse_program_ast"));
        assert!(assembly.contains(".globl parser_init"));
        assert!(assembly.contains(".globl next_token"));
        assert!(assembly.contains(".globl token_slice"));
        assert!(assembly.contains(".globl expression_arena_init"));
        assert!(assembly.contains(".globl expression_argument_add"));
        assert!(assembly.contains(".globl expression_tree"));
        assert!(assembly.contains(".globl statement_arena_init"));
        assert!(assembly.contains(".globl parse_program_tree"));
        assert!(assembly.contains(".globl semantic_init"));
        assert!(assembly.contains(".globl semantic_check_parser"));
        assert!(assembly.contains(".globl semantic_expression_type"));
        assert!(assembly.contains(".globl parse_type_kind"));
        assert!(assembly.contains(".globl semantic_collect_signatures"));
        assert!(assembly.contains(".globl semantic_function_index"));
        assert!(assembly.contains(".globl semantic_field_type"));
        assert!(assembly.contains(".globl ir_init"));
        assert!(assembly.contains(".globl ir_lower_expression"));
        assert!(assembly.contains(".globl ir_lower_program"));
        assert!(assembly.contains(".globl ir_block_storage"));
        assert!(assembly.contains(".globl ir_block_new"));
        assert!(assembly.contains(".globl ir_emit_branch"));
        assert!(assembly.contains(".globl ir_validate"));
    }

    #[test]
    fn compiles_the_shared_lexer_conformance_fixture() {
        let source = include_str!("../compiler/lexer_conformance.k");
        let assembly = compile_source(source).expect("lexer conformance fixture should compile");
        assert!(assembly.contains(".globl main"));
    }

    #[test]
    fn compiles_the_k_lexer_conformance_harness() {
        let source = include_str!("../compiler/lexer.k");
        let harness = include_str!("../compiler/lexer_conformance_harness.k");
        let combined = format!("{source}\n{harness}");
        let assembly =
            compile_source(&combined).expect("K lexer conformance harness should compile");
        assert!(assembly.contains(".globl lex_one"));
        assert!(assembly.contains(".globl lexer_conformance"));
        assert!(assembly.contains(".globl main"));
    }

    #[test]
    fn compiles_the_k_hello_world_program() {
        let source = include_str!("../compiler/hello.k");
        let assembly = compile_source(source).expect("hello world should compile");
        assert!(assembly.contains("mov rax, 1"));
        assert!(assembly.contains(".byte 72, 101, 108, 108, 111"));
    }

    #[test]
    fn compiles_the_k_backend_slice() {
        let source = include_str!("../compiler/backend.k");
        let assembly = compile_source(source).expect("K backend slice should compile");
        assert!(assembly.contains(".globl backend_init"));
        assert!(assembly.contains(".globl backend_function_label"));
        assert!(assembly.contains(".globl backend_binary"));
        assert!(assembly.contains(".globl backend_emit_instruction"));
        assert!(assembly.contains(".globl backend_integer"));
        assert!(assembly.contains(".globl backend_load_local"));
        assert!(assembly.contains(".globl backend_store_local"));
        assert!(assembly.contains(".globl backend_emit_block"));
        assert!(assembly.contains(".globl backend_emit_function"));
    }
}
