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
pub fn compile_source_for_target(source: &str, target: Target) -> Result<String, CompileError> {
    compile_sources_for_target(&[source], target)
}

/// Compile multiple K source buffers together into deterministic target assembly.
pub fn compile_sources(sources: &[&str]) -> Result<String, CompileError> {
    compile_sources_for_target(sources, Target::default())
}

/// Compile multiple K source buffers together for an explicit target.
pub fn compile_sources_for_target(sources: &[&str], target: Target) -> Result<String, CompileError> {
    if !target.is_implemented() {
        return Err(CompileError::UnsupportedTarget(target));
    }
    let mut parsed_programs = Vec::with_capacity(sources.len());
    for source in sources {
        let program = parser::parse(source).map_err(CompileError::Parse)?;
        parsed_programs.push(program);
    }
    let combined = combine_programs(&parsed_programs)?;
    sema::check(&combined).map_err(CompileError::Semantic)?;
    codegen::emit_for_target(&combined, target).map_err(CompileError::Codegen)
}

/// Combine multiple parsed K programs into a single compilation unit in deterministic order.
pub fn combine_programs(programs: &[parser::Program]) -> Result<parser::Program, CompileError> {
    if programs.is_empty() {
        return Ok(parser::Program {
            structs: Vec::new(),
            extern_functions: Vec::new(),
            functions: Vec::new(),
            span: crate::lexer::Span::new(0, 0),
        });
    }
    if programs.len() == 1 {
        return Ok(programs[0].clone());
    }

    let mut structs = Vec::new();
    let mut struct_names = std::collections::HashSet::new();
    let mut extern_functions = Vec::new();
    let mut extern_names = std::collections::HashSet::new();
    let mut functions = Vec::new();
    let mut function_names = std::collections::HashSet::new();

    for program in programs {
        for struct_def in &program.structs {
            if !struct_names.insert(struct_def.name.clone()) {
                return Err(CompileError::Semantic(vec![sema::SemanticError {
                    span: struct_def.span,
                    message: format!("duplicate struct `{}` in multi-file compilation", struct_def.name),
                }]));
            }
            structs.push(struct_def.clone());
        }

        for func in &program.functions {
            if !function_names.insert(func.name.clone()) {
                return Err(CompileError::Semantic(vec![sema::SemanticError {
                    span: func.span,
                    message: format!("duplicate function `{}` in multi-file compilation", func.name),
                }]));
            }
            functions.push(func.clone());
        }

        for extern_func in &program.extern_functions {
            if !extern_names.contains(&extern_func.name) {
                extern_names.insert(extern_func.name.clone());
                extern_functions.push(extern_func.clone());
            }
        }
    }

    // Filter out extern declarations if a concrete definition is provided
    extern_functions.retain(|ext| !function_names.contains(&ext.name));

    let span = crate::lexer::Span::new(
        programs.first().unwrap().span.start,
        programs.last().unwrap().span.end,
    );

    Ok(parser::Program {
        structs,
        extern_functions,
        functions,
        span,
    })
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
    fn compiles_arithmetic_program_through_the_complete_pipeline() {
        let source = "int main() { let answer = 20 + 22; return answer; }";
        let assembly = compile_source_for_target(source, Target::X86_64KrumpyOs).unwrap();

        let constant = assembly
            .find("  push 42\n")
            .expect("constant folding should emit the arithmetic result");
        let store = assembly
            .find("  mov QWORD PTR [rbp-8], rax\n")
            .expect("the let binding should store its value");
        let load = assembly
            .find("  mov rax, QWORD PTR [rbp-8]\n")
            .expect("the return should load the local value");
        let return_value = assembly[load..]
            .find("  pop rax\n")
            .map(|offset| load + offset)
            .expect("the return should consume the stack value");

        assert!(constant < store);
        assert!(store < load);
        assert!(load < return_value);
        assert!(!assembly.contains(".globl _start"));
    }

    #[test]
    fn returns_structured_diagnostics() {
        let error = compile_source("int main() { return missing; }").unwrap_err();
        assert!(matches!(error, CompileError::Semantic(_)));
        assert!(error.to_string().contains("undefined name"));
    }

    #[test]
    fn rejects_unimplemented_targets_before_codegen() {
        let error = compile_source_for_target("int main() { return 0; }", Target::Aarch64KrumpyOs)
            .unwrap_err();
        assert!(matches!(
            error,
            CompileError::UnsupportedTarget(Target::Aarch64KrumpyOs)
        ));
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
        assert!(source.contains("int ir_integer_literal("));
        assert!(source.contains("if (node.kind == 7)"));
        assert!(source.contains("ir.instructions[ir.instruction_count - 1].extra = 8"));
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
        assert!(assembly.contains(".globl backend_constant"));
        assert!(assembly.contains(".globl backend_function_label"));
        assert!(assembly.contains(".globl backend_binary"));
        assert!(assembly.contains(".globl backend_call"));
        assert!(assembly.contains(".globl backend_parameter_store"));
        assert!(assembly.contains(".globl backend_branch"));
        assert!(assembly.contains(".globl backend_jump"));
        assert!(assembly.contains(".globl backend_block_label"));
        assert!(source.contains("if (operator == 10)"));
        assert!(source.contains("if (instruction.kind == 12)"));
        assert!(source.contains("if (instruction.kind == 13)"));
        assert!(assembly.contains(".globl backend_emit_instruction"));
        assert!(assembly.contains(".globl backend_integer"));
        assert!(assembly.contains(".globl backend_load_local"));
        assert!(assembly.contains(".globl backend_store_local"));
        assert!(assembly.contains(".globl backend_address_local"));
        assert!(assembly.contains(".globl backend_field_address"));
        assert!(assembly.contains(".globl backend_load"));
        assert!(assembly.contains(".globl backend_scale"));
        assert!(assembly.contains(".globl backend_store"));
        assert!(assembly.contains(".globl backend_emit_block"));
        assert!(assembly.contains(".globl backend_emit_function"));
        assert!(source.contains("if (instruction.kind == 1)"));
        assert!(source.contains("if (instruction.kind == 15)"));
        assert!(source.contains("return backend_constant("));
    }

    #[test]
    fn compiles_the_k_compiler_driver() {
        let parser = include_str!("../compiler/parser.k");
        let backend = include_str!("../compiler/backend.k");
        let driver = include_str!("../compiler/driver.k");
        let source = format!("{parser}\n{backend}\n{driver}");
        let assembly = compile_source(&source).expect("K compiler driver should compile");
        assert!(assembly.contains(".globl compiler_init"));
        assert!(assembly.contains(".globl compiler_frontend"));
        assert!(assembly.contains(".globl compiler_emit_backend"));
        assert!(source.contains("parse_program_tree"));
        assert!(source.contains("semantic_check_parser"));
        assert!(source.contains("ir_lower_program"));
        assert!(source.contains("ir_validate"));
        assert!(source.contains("backend_emit_function"));
    }

    #[test]
    fn compiles_multi_file_program_reproducibly() {
        let file1 = "struct Point { int x; int y; } extern int get_y(struct Point* p); int get_x(struct Point* p) { return p.x; }";
        let file2 = "struct Point { int x; int y; } int get_y(struct Point* p) { return p.y; } int main() { struct Point pt; pt.x = 10; pt.y = 20; return get_x(&pt) + get_y(&pt); }";
        let assembly = super::compile_sources(&[file1, file2]).expect("multi-file compilation should succeed");
        assert!(assembly.contains(".globl get_x"));
        assert!(assembly.contains(".globl get_y"));
        assert!(assembly.contains(".globl main"));
    }

    #[test]
    fn compiles_compiler_modules_via_multi_file() {
        let lexer = include_str!("../compiler/lexer.k");
        let parser = include_str!("../compiler/parser.k");
        let backend = include_str!("../compiler/backend.k");
        let driver = include_str!("../compiler/driver.k");
        let main = include_str!("../compiler/main.k");
        let assembly = super::compile_sources(&[parser, backend, driver, main]).expect("multi-file compiler should compile");
        assert!(assembly.contains(".globl compiler_init"));
        assert!(assembly.contains(".globl compiler_emit_backend"));
        assert!(assembly.contains(".globl main"));
        let _ = lexer;
    }
}
