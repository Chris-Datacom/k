//! x86-64 System V code generation from K's typed IR.

use std::collections::{BTreeMap, HashMap};
use std::fmt;

use crate::ir::{BasicBlock, Instruction, IrFunction, IrType, TypedProgram};
use crate::parser::{BinaryOperator, Program, UnaryOperator};
use crate::target::Target;

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
    emit_for_target(program, Target::default())
}

pub fn emit_for_target(program: &Program, target: Target) -> Result<String, CodegenError> {
    let typed = crate::ir::lower(program).map_err(|errors| CodegenError {
        message: errors
            .into_iter()
            .map(|error| error.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
    })?;
    emit_typed_for_target(&typed, target)
}

pub fn emit_typed(program: &TypedProgram) -> Result<String, CodegenError> {
    emit_typed_for_target(program, Target::default())
}

pub fn emit_typed_for_target(
    program: &TypedProgram,
    target: Target,
) -> Result<String, CodegenError> {
    let mut generator = Generator {
        output: String::from(".intel_syntax noprefix\n.text\n"),
        rodata: String::new(),
        label: 0,
        struct_sizes: program
            .structs
            .iter()
            .map(|(name, layout)| (name.clone(), layout.size))
            .collect(),
        target,
    };
    for (name, function) in &program.functions {
        generator.function(name, function)?;
    }
    if target == Target::X86_64SystemV && program.functions.contains_key("main") {
        generator.output.push_str(
            ".globl _start\n.type _start, @function\n_start:\n  call main\n  mov rdi, rax\n  mov rax, 60\n  syscall\n\n",
        );
    }
    if generator.rodata.is_empty() {
        Ok(generator.output)
    } else {
        Ok(format!(
            "{}\n.section .rodata\n{}",
            generator.output, generator.rodata
        ))
    }
}

struct Generator {
    output: String,
    rodata: String,
    label: usize,
    struct_sizes: BTreeMap<String, i64>,
    target: Target,
}

fn ir_size(ty: &IrType, structs: &BTreeMap<String, i64>) -> i32 {
    match ty {
        IrType::Struct(name) => structs.get(name).copied().unwrap_or(8) as i32,
        _ => 8,
    }
}

fn memory_width(ty: &IrType) -> (&'static str, &'static str, &'static str, &'static str) {
    match ty {
        IrType::Char | IrType::U8 | IrType::Bool => ("BYTE", "eax", "al", "movzx"),
        IrType::U16 => ("WORD", "eax", "ax", "movzx"),
        IrType::U32 => ("DWORD", "eax", "eax", "mov"),
        IrType::I32 => ("DWORD", "rax", "eax", "movsxd"),
        IrType::U64 | IrType::I64 | IrType::Int | IrType::Pointer(_, _) | IrType::Struct(_) => {
            ("QWORD", "rax", "rax", "mov")
        }
        IrType::Void => ("QWORD", "rax", "rax", "mov"),
    }
}

fn emit_load(output: &mut String, address: &str, ty: &IrType) {
    let (width, load_register, _, extension) = memory_width(ty);
    output.push_str(&format!(
        "  {extension} {load_register}, {width} PTR {address}\n  push rax\n"
    ));
}

fn emit_store(output: &mut String, address: &str, ty: &IrType) {
    let (width, _, store_register, _) = memory_width(ty);
    output.push_str(&format!("  mov {width} PTR {address}, {store_register}\n"));
}

impl Generator {
    fn function(&mut self, name: &str, function: &IrFunction) -> Result<(), CodegenError> {
        let mut slots = HashMap::new();
        let mut frame_size = 0i32;
        for local in &function.locals {
            frame_size += ir_size(&local.ty, &self.struct_sizes);
            slots.insert(local.name.clone(), frame_size);
        }
        let return_label = self.fresh_label("return");
        self.output.push_str(&format!(
            ".globl {name}\n.type {name}, @function\n{name}:\n  push rbp\n  mov rbp, rsp\n"
        ));
        if frame_size != 0 {
            self.output.push_str(&format!("  sub rsp, {frame_size}\n"));
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
            self.output.push_str(&format!("  mov rax, {register}\n"));
            let address = format!("[rbp-{}]", slots[&local.name]);
            emit_store(&mut self.output, &address, &local.ty);
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
                self.output
                    .push_str(&format!("{}:\n", block_labels[block.id]));
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
                    self.output
                        .push_str(&format!("  lea rax, {label}[rip]\n  push rax\n"));
                }
                Instruction::PrintString(value) => {
                    if self.target != Target::X86_64SystemV {
                        return Err(
                            self.error("print is only available on the Linux System V target")
                        );
                    }
                    let label = self.fresh_label("string");
                    self.rodata.push_str(&format!("{label}:\n  .byte "));
                    for (index, byte) in value.iter().enumerate() {
                        if index != 0 {
                            self.rodata.push_str(", ");
                        }
                        self.rodata.push_str(&byte.to_string());
                    }
                    self.rodata.push('\n');
                    self.output.push_str(&format!(
                        "  mov rax, 1\n  mov rdi, 1\n  lea rsi, {label}[rip]\n  mov rdx, {}\n  syscall\n",
                        value.len()
                    ));
                }
                Instruction::LoadLocal { name, ty } => {
                    let slot = slots.get(name).ok_or_else(|| self.error("unknown local"))?;
                    let address = format!("[rbp-{slot}]");
                    emit_load(&mut self.output, &address, ty);
                }
                Instruction::StoreLocal { name, ty } => {
                    self.output.push_str("  pop rax\n");
                    let address = format!(
                        "[rbp-{}]",
                        slots.get(name).ok_or_else(|| self.error("unknown local"))?
                    );
                    emit_store(&mut self.output, &address, ty);
                }
                Instruction::AddressLocal { name, .. } => self.output.push_str(&format!(
                    "  lea rax, [rbp-{}]\n  push rax\n",
                    slots.get(name).ok_or_else(|| self.error("unknown local"))?
                )),
                Instruction::FieldAddress { offset, .. } => {
                    self.output.push_str("  pop rax\n");
                    if *offset == 0 {
                        self.output.push_str("  push rax\n");
                    } else {
                        self.output
                            .push_str(&format!("  add rax, {offset}\n  push rax\n"));
                    }
                }
                Instruction::Load { ty, volatile } => {
                    self.output.push_str("  pop rax\n");
                    if *volatile {
                        self.output.push_str("  ; volatile load\n");
                    }
                    emit_load(&mut self.output, "[rax]", ty);
                }
                Instruction::Scale { bytes } => {
                    self.output
                        .push_str(&format!("  pop rax\n  imul rax, {bytes}\n  push rax\n"));
                }
                Instruction::Store { ty, volatile } => {
                    self.output.push_str("  pop rax\n  pop rdi\n");
                    if *volatile {
                        self.output.push_str("  ; volatile store\n");
                    }
                    let (width, _, register, _) = memory_width(ty);
                    self.output
                        .push_str(&format!("  mov {width} PTR [rdi], {register}\n"));
                }
                Instruction::Unary {
                    operator,
                    ty,
                    volatile,
                } => match operator {
                    UnaryOperator::Negate => {
                        self.output.push_str("  pop rax\n  neg rax\n  push rax\n")
                    }
                    UnaryOperator::BitwiseNot => {
                        self.output.push_str("  pop rax\n  not rax\n  push rax\n")
                    }
                    UnaryOperator::Dereference => {
                        self.output.push_str("  pop rax\n");
                        if *volatile {
                            self.output.push_str("  ; volatile load\n");
                        }
                        emit_load(&mut self.output, "[rax]", ty);
                    }
                    UnaryOperator::AddressOf => {
                        return Err(self.error("address-of must be lowered as an address"))
                    }
                },
                Instruction::Binary { operator, .. } => {
                    self.output.push_str("  pop rdi\n  pop rax\n");
                    self.binary(*operator);
                    self.output.push_str("  push rax\n");
                }
                Instruction::Call {
                    name, arguments, ..
                } => {
                    const REGISTERS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
                    if *arguments > REGISTERS.len() {
                        return Err(
                            self.error("calls with more than six arguments are not supported")
                        );
                    }
                    for register in REGISTERS[..*arguments].iter().rev() {
                        self.output.push_str(&format!("  pop {register}\n"));
                    }
                    self.output
                        .push_str(&format!("  call {name}\n  push rax\n"));
                }
                Instruction::OutB => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self
                            .error("outb is only available on the freestanding krumpyos target"));
                    }
                    self.output.push_str("  pop rax\n  pop rdx\n  out dx, al\n");
                }
                Instruction::InB => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(
                            self.error("inb is only available on the freestanding krumpyos target")
                        );
                    }
                    self.output
                        .push_str("  pop rdx\n  in al, dx\n  movzx eax, al\n  push rax\n");
                }
                Instruction::ReadCr0 => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("read_cr0 is only available on krumpyos target"));
                    }
                    self.output.push_str("  mov rax, cr0\n  push rax\n");
                }
                Instruction::WriteCr0 => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("write_cr0 is only available on krumpyos target"));
                    }
                    self.output.push_str("  pop rax\n  mov cr0, rax\n");
                }
                Instruction::ReadCr2 => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("read_cr2 is only available on krumpyos target"));
                    }
                    self.output.push_str("  mov rax, cr2\n  push rax\n");
                }
                Instruction::ReadCr3 => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("read_cr3 is only available on krumpyos target"));
                    }
                    self.output.push_str("  mov rax, cr3\n  push rax\n");
                }
                Instruction::WriteCr3 => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("write_cr3 is only available on krumpyos target"));
                    }
                    self.output.push_str("  pop rax\n  mov cr3, rax\n");
                }
                Instruction::ReadCr4 => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("read_cr4 is only available on krumpyos target"));
                    }
                    self.output.push_str("  mov rax, cr4\n  push rax\n");
                }
                Instruction::WriteCr4 => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("write_cr4 is only available on krumpyos target"));
                    }
                    self.output.push_str("  pop rax\n  mov cr4, rax\n");
                }
                Instruction::Lidt => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("lidt is only available on krumpyos target"));
                    }
                    self.output.push_str("  pop rax\n  lidt [rax]\n");
                }
                Instruction::Sidt => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("sidt is only available on krumpyos target"));
                    }
                    self.output.push_str("  pop rax\n  sidt [rax]\n");
                }
                Instruction::Invlpg => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("invlpg is only available on krumpyos target"));
                    }
                    self.output.push_str("  pop rax\n  invlpg [rax]\n");
                }
                Instruction::Rdmsr => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("rdmsr is only available on krumpyos target"));
                    }
                    self.output.push_str("  pop rcx\n  rdmsr\n  shl rdx, 32\n  or rax, rdx\n  push rax\n");
                }
                Instruction::Wrmsr => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error("wrmsr is only available on krumpyos target"));
                    }
                    self.output.push_str("  pop rax\n  pop rdx\n  mov rcx, rax\n  mov rax, rdx\n  mov rdx, rdx\n  shr rdx, 32\n  wrmsr\n");
                }
                instruction @ (Instruction::Cli
                | Instruction::Sti
                | Instruction::Hlt
                | Instruction::Pause) => {
                    if self.target != Target::X86_64KrumpyOs {
                        return Err(self.error(
                            "privileged interrupt intrinsics (cli, sti, hlt, and pause) are only available on the freestanding krumpyos target",
                        ));
                    }
                    let mnemonic = match instruction {
                        Instruction::Cli => "cli",
                        Instruction::Sti => "sti",
                        Instruction::Hlt => "hlt",
                        Instruction::Pause => "pause",
                        _ => unreachable!(),
                    };
                    self.output.push_str(&format!("  {mnemonic}\n"));
                }
                Instruction::Cast(ty) => {
                    self.output.push_str("  pop rax\n");
                    match ty {
                        IrType::Char | IrType::U8 | IrType::Bool => {
                            self.output.push_str("  movzx eax, al\n")
                        }
                        IrType::U16 => self.output.push_str("  movzx eax, ax\n"),
                        IrType::U32 => self.output.push_str("  mov eax, eax\n"),
                        IrType::I32 => self.output.push_str("  movsxd rax, eax\n"),
                        IrType::U64
                        | IrType::I64
                        | IrType::Int
                        | IrType::Pointer(_, _)
                        | IrType::Struct(_)
                        | IrType::Void => {}
                    }
                    self.output.push_str("  push rax\n");
                }
                Instruction::Pop => self.output.push_str("  add rsp, 8\n"),
                Instruction::Branch {
                    then_block,
                    else_block,
                } => self.output.push_str(&format!(
                    "  pop rax\n  test rax, rax\n  jne {}\n  jmp {}\n",
                    labels[*then_block], labels[*else_block]
                )),
                Instruction::Jump { target } => self
                    .output
                    .push_str(&format!("  jmp {}\n", labels[*target])),
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
            BinaryOperator::BitwiseAnd => self.output.push_str("  and rax, rdi\n"),
            BinaryOperator::BitwiseOr => self.output.push_str("  or rax, rdi\n"),
            BinaryOperator::BitwiseXor => self.output.push_str("  xor rax, rdi\n"),
            BinaryOperator::ShiftLeft => self.output.push_str("  mov rcx, rdi\n  shl rax, cl\n"),
            BinaryOperator::ShiftRight => self.output.push_str("  mov rcx, rdi\n  shr rax, cl\n"),
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
                self.output.push_str(&format!(
                    "  cmp rax, rdi\n  set{condition} al\n  movzx rax, al\n"
                ));
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
    use super::{emit, emit_for_target};
    use crate::parser::parse;
    use crate::target::Target;

    #[test]
    fn emits_from_ir_for_arithmetic_and_return() {
        let program = parse("int main() { let answer = 40 + 2; return answer; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("push 42"));
        assert!(assembly.contains("mov QWORD PTR [rbp-8], rax"));
    }

    #[test]
    fn emits_ir_control_flow_and_memory() {
        let program =
            parse("int read(int* ptr) { while (true) { ptr[1] = 7; return *ptr; } return 0; }")
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
    fn emits_linux_start_wrapper_for_main() {
        let program = parse("int main() { return 0; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains(".globl _start"));
        assert!(assembly.contains("call main"));
        assert!(assembly.contains("mov rax, 60"));
    }

    #[test]
    fn emits_kernel_interrupt_intrinsics_only_for_krumpyos() {
        let program = parse("void main() { cli(); sti(); pause(); hlt(); }").unwrap();
        let assembly = emit_for_target(&program, Target::X86_64KrumpyOs).unwrap();
        assert!(assembly.contains("  cli\n"));
        assert!(assembly.contains("  sti\n"));
        assert!(assembly.contains("  pause\n"));
        assert!(assembly.contains("  hlt\n"));

        let error = emit_for_target(&program, Target::X86_64SystemV).unwrap_err();
        assert!(error
            .to_string()
            .contains("privileged interrupt intrinsics"));
    }

    #[test]
    fn scales_char_pointers_by_one() {
        let program = parse("char read(char* ptr) { return ptr[1]; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("imul rax, 1"));
    }

    #[test]
    fn emits_struct_field_offsets_for_reads_and_writes() {
        let program = parse(
            "struct Token { int kind; int start; } int read(struct Token* token) { token.start = 42; return token.start; }",
        )
        .unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("add rax, 8"));
        assert!(assembly.contains("mov QWORD PTR [rdi], rax"));
    }

    #[test]
    fn emits_fixed_width_local_accesses() {
        let program = parse("u8 read(u8 value) { let copy = value; return copy; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("movzx eax, BYTE PTR [rbp-16]"));
        assert!(assembly.contains("mov BYTE PTR [rbp-8], al"));
    }

    #[test]
    fn emits_fixed_width_pointer_accesses() {
        let program = parse("u16 read(u16* ptr) { return ptr[1]; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("imul rax, 2"));
        assert!(assembly.contains("movzx eax, WORD PTR [rax]"));
    }

    #[test]
    fn reserves_the_complete_size_of_local_structs() {
        let program = parse(
            "struct Token { char kind; int start; int end; } int main() { struct Token token; return 0; }",
        )
        .unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("sub rsp, 17"));
    }

    #[test]
    fn krumpyos_intrinsics_emit_expected_instructions() {
        let program = parse(
            "void write_serial(u16 port, u8 value) { cli(); outb(port, value); sti(); pause(); hlt(); } u8 read_serial(u16 port) { return inb(port); }",
        )
        .unwrap();
        let assembly = emit_for_target(&program, Target::X86_64KrumpyOs).unwrap();
        assert!(assembly.contains("cli"));
        assert!(assembly.contains("out dx, al"));
        assert!(assembly.contains("sti"));
        assert!(assembly.contains("pause"));
        assert!(assembly.contains("hlt"));
        assert!(assembly.contains("in al, dx"));
    }

    #[test]
    fn krumpyos_intrinsics_are_rejected_on_hosted_targets() {
        let program =
            parse("void write_serial(u16 port, u8 value) { outb(port, value); }").unwrap();
        let error = emit_for_target(&program, Target::X86_64SystemV).unwrap_err();
        assert!(error.message.contains("krumpyos"));

        let program = parse("u8 read_serial(u16 port) { return inb(port); }").unwrap();
        let error = emit_for_target(&program, Target::X86_64SystemV).unwrap_err();
        assert!(error.message.contains("krumpyos"));

        let program = parse("void idle() { cli(); }").unwrap();
        let error = emit_for_target(&program, Target::X86_64SystemV).unwrap_err();
        assert!(error.message.contains("krumpyos"));

        let program = parse("void idle() { sti(); }").unwrap();
        let error = emit_for_target(&program, Target::X86_64SystemV).unwrap_err();
        assert!(error.message.contains("krumpyos"));

        let program = parse("void idle() { hlt(); }").unwrap();
        let error = emit_for_target(&program, Target::X86_64SystemV).unwrap_err();
        assert!(error.message.contains("krumpyos"));

        let program = parse("void idle() { pause(); }").unwrap();
        let error = emit_for_target(&program, Target::X86_64SystemV).unwrap_err();
        assert!(error.message.contains("krumpyos"));
    }

    #[test]
    fn emits_no_instructions_for_same_width_pointer_integer_casts() {
        let program = parse("u32* mmio(u64 address) { return (u32*)address; }").unwrap();
        let assembly = emit(&program).unwrap();
        // The value is already a full 64-bit quantity; casting between
        // pointers and 64-bit integers is a pure type-system operation with
        // no runtime cost.
        assert!(!assembly.contains("movzx"));
        assert!(!assembly.contains("movsxd"));
    }

    #[test]
    fn emits_truncation_for_narrowing_casts() {
        let program = parse("u8 truncate(u64 value) { return (u8)value; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("movzx eax, al"));
    }

    #[test]
    fn emits_sign_extension_for_narrowing_i32_casts() {
        let program = parse("i32 truncate(u64 value) { return (i32)value; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("movsxd rax, eax"));
    }

    #[test]
    fn marks_volatile_dereference_loads_and_stores_in_assembly() {
        let program = parse(
            "void poke(volatile u16* port) { *port = (u16)1; } u16 peek(volatile u16* port) { return *port; }",
        )
        .unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("; volatile store"));
        assert!(assembly.contains("; volatile load"));
    }

    #[test]
    fn marks_volatile_indexed_loads_and_stores_in_assembly() {
        let program = parse(
            "void write(volatile u16* buffer, int index) { buffer[index] = (u16)1; } u16 read(volatile u16* buffer, int index) { return buffer[index]; }",
        )
        .unwrap();
        let assembly = emit(&program).unwrap();
        assert!(assembly.contains("; volatile store"));
        assert!(assembly.contains("; volatile load"));
    }

    #[test]
    fn does_not_mark_plain_pointer_accesses_as_volatile_in_assembly() {
        let program = parse("u16 read(u16* ptr) { return ptr[1]; }").unwrap();
        let assembly = emit(&program).unwrap();
        assert!(!assembly.contains("volatile"));
    }
}
