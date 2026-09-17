//! x86-64 System V code generation from K's typed IR.

use std::collections::HashMap;
use std::fmt;

use crate::ir::{BasicBlock, Instruction, IrFunction, TypedProgram};
use crate::parser::{BinaryOperator, Program, UnaryOperator};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CodegenError {
    pub message: String,
}

impl fmt::Display for CodegenError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "code generation error: {}", self.message)
    }
}

pub fn emit(program: &Program) -> Result<String, CodegenError> {
    let typed = crate::ir::lower(program).map_err(|errors| CodegenError {
        message: errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    })?;
    emit_typed(&typed)
}

pub fn emit_typed(program: &TypedProgram) -> Result<String, CodegenError> {
    let mut generator = Generator {
        output: String::from(".intel_syntax noprefix\n.text\n"),
        rodata: String::new(),
        label: 0,
    };
    for (name, function) in &program.functions {
        generator.function(name, function)?;
    }
    if generator.rodata.is_empty() {
        Ok(generator.output)
    } else {
        Ok(format!("{}\n.section .rodata\n{}", generator.output, generator.rodata))
    }
}

struct Generator {
    output: String,
    rodata: String,
    label: usize,
}

impl Generator {
    fn function(&mut self, name: &str, function: &IrFunction) -> Result<(), CodegenError> {
        let mut slots = HashMap::new();
        for (index, local) in function.locals.iter().enumerate() {
            slots.insert(local.name.clone(), (index as i32 + 1) * 8);
        }
        let return_label = self.fresh_label("return");
        self.output.push_str(&format!(
            ".globl {name}\n.type {name}, @function\n{name}:\n  push rbp\n  mov rbp, rsp\n"
        ));
        if !function.locals.is_empty() {
            self.output
                .push_str(&format!("  sub rsp, {}\n", function.locals.len() * 8));
        }

        const ARGUMENT_REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        for (index, parameter) in function.parameters.iter().enumerate() {
            let local = function
                .locals
                .get(index)
                .ok_or_else(|| self.error("missing parameter local"))?;
            let register = ARGUMENT_REGISTERS
                .get(index)
                .ok_or_else(|| self.error("more than six parameters are not supported"))?;
            self.output.push_str(&format!(
                "  mov QWORD PTR [rbp-{}], {}\n",
                slots[&local.name], register
            ));
            let _ = parameter;
        }

        let mut block_labels = Vec::new();
        for block in &function.blocks {
            let label = if block.id == 0 {
                name.to_owned()
            } else {
                self.fresh_label("block")
            };
            block_labels.push(label);
        }
        for block in &function.blocks {
            if block.id != 0 {
                self.output.push_str(&format!("{}:\n", block_labels[block.id]));
            }
            self.block(block, &block_labels, &slots, &return_label)?;
        }
        self.output.push_str(&format!(
            "  mov rax, 0\n{return_label}:\n  leave\n  ret\n\n"
        ));
        Ok(())
    }

    fn block(
        &mut self,
        block: &BasicBlock,
        labels: &[String],
        slots: &HashMap<String, i32>,
        return_label: &str,
    ) -> Result<(), CodegenError> {
        for instruction in &block.instructions {
            match instruction {
                Instruction::Constant(value) => self.output.push_str(&format!("  push {value}\n")),
                Instruction::StringLiteral(value) => {
                    let label = self.fresh_label("string");
                    self.rodata.push_str(&format!("{label}:\n  .byte "));
                    for (index, byte) in value.iter().chain(std::iter::once(&0)).enumerate() {
                        if index != 0 {
                            self.rodata.push_str(", ");
                        }
                        self.rodata.push_str(&byte.to_string());
                    }
                    self.rodata.push('\n');
                    self.output.push_str(&format!("  lea rax, {label}[rip]\n  push rax\n"));
                }
                Instruction::LoadLocal { name, .. } => self.output.push_str(&format!(
                    "  push QWORD PTR [rbp-{}]\n",
                    slots
                        .get(name)
                        .ok_or_else(|| self.error("unknown local"))?
                )),
                Instruction::StoreLocal { name, .. } => self.output.push_str(&format!(
                    "  pop rax\n  mov QWORD PTR [rbp-{}], rax\n",
                    slots
                        .get(name)
                        .ok_or_else(|| self.error("unknown local"))?
                )),
                Instruction::AddressLocal { name, .. } => self.output.push_str(&format!(
                    "  lea rax, [rbp-{}]\n  push rax\n",
                    slots
                        .get(name)
                        .ok_or_else(|| self.error("unknown local"))?
                )),
                Instruction::Load { .. } => {
                    self.output
                        .push_str("  pop rax\n  mov rax, QWORD PTR [rax]\n  push rax\n");
                }
                Instruction::Scale { bytes } => {
                    self.output.push_str(&format!("  pop rax\n  imul rax, {bytes}\n  push rax\n"));
                }
                Instruction::Store { .. } => {
                    self.output
                        .push_str("  pop rax\n  pop rdi\n  mov QWORD PTR [rdi], rax\n");
                }
                Instruction::Unary { operator, .. } => match operator {
                    UnaryOperator::Negate => {
                        self.output.push_str("  pop rax\n  neg rax\n  push rax\n")
                    }
                    UnaryOperator::Dereference => self
                        .output
                        .push_str("  pop rax\n  mov rax, QWORD PTR [rax]\n  push rax\n"),
                    UnaryOperator::AddressOf => {
                        return Err(self.error("address-of must be lowered as an address"))
                    }
                },
                Instruction::Binary { operator, .. } => {
                    self.output.push_str("  pop rdi\n  pop rax\n");
                    self.binary(*operator);
                    self.output.push_str("  push rax\n");
                }
                Instruction::Call { name, arguments, .. } => {
                    const REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                    if *arguments > REGISTERS.len() {
                        return Err(self.error("calls with more than six arguments are not supported"));
                    }
                    for register in REGISTERS[..*arguments].iter().rev() {
                        self.output.push_str(&format!("  pop {register}\n"));
                    }
                    self.output.push_str(&format!("  call {name}\n  push rax\n"));
                }
                Instruction::Pop => self.output.push_str("  add rsp, 8\n"),
                Instruction::Branch {
                    then_block,
                    else_block,
                } => self.output.push_str(&format!(
                    "  pop rax\n  test rax, rax\n  jne {}\n  jmp {}\n",
                    labels[*then_block], labels[*else_block]
                )),
                Instruction::Jump { target } => {
                    self.output.push_str(&format!("  jmp {}\n", labels[*target]))
                }
                Instruction::Return { has_value } => {
                    if *has_value {
                        self.output.push_str("  pop rax\n");
                    }
                    self.output.push_str(&format!("  jmp {return_label}\n"));
                }
            }
        }
        Ok(())
    }

    fn binary(&mut self, operator: BinaryOperator) {
        match operator {
            BinaryOperator::Add => self.output.push_str("  add rax, rdi\n"),
            BinaryOperator::Subtract => self.output.push_str("  sub rax, rdi\n"),
            BinaryOperator::Multiply => self.output.push_str("  imul rax, rdi\n"),
            BinaryOperator::Divide => self.output.push_str("  cqo\n  idiv rdi\n"),
            BinaryOperator::Equal
            | BinaryOperator::NotEqual
            | BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => {
                let condition = match operator {
                    BinaryOperator::Equal => "e",
                    BinaryOperator::NotEqual => "ne",
                    BinaryOperator::Less => "l",
                    BinaryOperator::LessEqual => "le",
                    BinaryOperator::Greater => "g",
                    BinaryOperator::GreaterEqual => "ge",
                    _ => unreachable!(),
                };
                self.output
                    .push_str(&format!("  cmp rax, rdi\n  set{condition} al\n  movzx rax, al\n"));
            }
        }
    }

    fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!(".L{prefix}_{}", self.label);
        self.label += 1;
        label
    }

    fn error(&self, message: &str) -> CodegenError {
        CodegenError {
            message: message.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::emit;
    use crate::parser::parse;

    #[test]
    fn emits_from_ir_for_arithmetic_and_return() {
        let program = parse("int main() { let answer = 40 + 2; return answer; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("push 42"));
        assert!(assembly.contains("mov QWORD PTR [rbp-8], rax"));
    }

    #[test]
    fn emits_ir_control_flow_and_memory() {
        let program = parse(
            "int read(int* ptr) { while (true) { ptr[1] = 7; return *ptr; } return 0; }",
        )
        .unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("imul rax, 8"));
        assert!(assembly.contains("mov QWORD PTR [rdi], rax"));
        assert!(assembly.contains("jmp .Lblock_"));
    }

    #[test]
    fn emits_string_data_and_byte_pointer_indexing() {
        let program = parse("char* text() { return \"hi\"; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains(".section .rodata"));
        assert!(assembly.contains(".byte 104, 105, 0"));
        assert!(assembly.contains("lea rax, .Lstring_"));
    }

    #[test]
    fn scales_char_pointers_by_one() {
        let program = parse("char read(char* ptr) { return ptr[1]; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("imul rax, 1"));
    }
}
