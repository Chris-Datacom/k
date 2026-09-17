// Caller-owned x86-64 assembly output primitives for the K-written backend.

struct BackendOutput {
    char* data;
    int capacity;
    int length;
    int error;
}

struct BackendInstruction {
    int kind;
    int operator;
    int type_kind;
    int start;
    int end;
    int left;
    int right;
    int extra;
    char* name;
    int name_length;
}

// These records mirror the caller-owned IR ranges in parser.k.  A local
// instruction uses extra as its already-resolved frame slot; name resolution
// remains a frontend concern.
struct BackendBlock {
    int start;
    int count;
    int id;
}

struct BackendFunction {
    int start;
    int count;
    int return_type;
    int parameter_start;
    int parameter_count;
    int block_start;
    int block_count;
}

struct BackendProgram {
    char* scratch;
    int scratch_capacity;
    struct BackendInstruction* instructions;
    int instruction_capacity;
    struct BackendBlock* blocks;
    int block_capacity;
}

int backend_init(
    struct BackendOutput* output,
    char* data,
    int capacity
) {
    output.data = data;
    output.capacity = capacity;
    output.length = 0;
    output.error = 0;
    return 1;
}

int backend_char(
    struct BackendOutput* output,
    char value
) {
    if (output.length >= output.capacity) {
        output.error = 1;
        return 0;
    }
    output.data[output.length] = value;
    output.length = output.length + 1;
    return 1;
}

int backend_integer(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int value
) {
    let negative = 0;
    if (value < 0) {
        negative = 1;
        value = 0 - value;
    }
    if (value == 0) {
        return backend_char(output, '0');
    }
    let count = 0;
    let digits = "0123456789";
    while (value > 0) {
        if (count >= scratch_capacity) {
            output.error = 5;
            return 0;
        }
        let quotient = value / 10;
        let remainder = value - (quotient * 10);
        scratch[count] = digits[remainder];
        count = count + 1;
        value = quotient;
    }
    if (negative == 1) {
        if (backend_char(output, '-') == 0) {
            return 0;
        }
    }
    count = count - 1;
    while (count >= 0) {
        if (backend_char(output, scratch[count]) == 0) {
            return 0;
        }
        count = count - 1;
    }
    return 1;
}

int backend_text(
    struct BackendOutput* output,
    char* text,
    int length
) {
    if (length < 0) {
        output.error = 2;
        return 0;
    }
    let index = 0;
    while (index < length) {
        if (backend_char(output, text[index]) == 0) {
            return 0;
        }
        index = index + 1;
    }
    return 1;
}

int backend_header(struct BackendOutput* output) {
    return backend_text(output, ".intel_syntax noprefix\n.text\n", 29);
}

int backend_function_label(
    struct BackendOutput* output,
    char* name,
    int name_length
) {
    if (backend_text(output, ".globl ", 7) == 0) {
        return 0;
    }
    if (backend_text(output, name, name_length) == 0) {
        return 0;
    }
    if (backend_text(output, "\n.type ", 7) == 0) {
        return 0;
    }
    if (backend_text(output, name, name_length) == 0) {
        return 0;
    }
    if (backend_text(output, ", @function\n", 12) == 0) {
        return 0;
    }
    return 1;
}

int backend_function_prologue(struct BackendOutput* output) {
    return backend_text(
        output,
        ":\n  push rbp\n  mov rbp, rsp\n",
        28
    );
}

int backend_return_zero(struct BackendOutput* output) {
    return backend_text(output, "  mov rax, 0\n  leave\n  ret\n", 27);
}

int backend_return(struct BackendOutput* output, int has_value) {
    if (has_value < 0) {
        output.error = 3;
        return 0;
    }
    if (has_value > 1) {
        output.error = 3;
        return 0;
    }
    if (has_value == 1) {
        return backend_text(output, "  pop rax\n  leave\n  ret\n", 24);
    }
    return backend_return_zero(output);
}

int backend_load_local(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int slot,
    int type_kind
) {
    if (slot < 0) {
        output.error = 6;
        return 0;
    }
    if (backend_text(output, "  mov ", 6) == 0) {
        return 0;
    }
    if (type_kind == 2) {
        if (backend_text(output, "zx eax, BYTE PTR [rbp-", 22) == 0) {
            return 0;
        }
    } else {
        if (backend_text(output, "rax, QWORD PTR [rbp-", 20) == 0) {
            return 0;
        }
    }
    if (backend_integer(output, scratch, scratch_capacity, slot) == 0) {
        return 0;
    }
    if (backend_text(output, "]\n  push rax\n", 13) == 0) {
        return 0;
    }
    return 1;
}

int backend_store_local(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int slot,
    int type_kind
) {
    if (slot < 0) {
        output.error = 6;
        return 0;
    }
    if (backend_text(output, "  pop rax\n  mov ", 16) == 0) {
        return 0;
    }
    if (type_kind == 2) {
        if (backend_text(output, "BYTE PTR [rbp-", 14) == 0) {
            return 0;
        }
        if (backend_integer(output, scratch, scratch_capacity, slot) == 0) {
            return 0;
        }
        return backend_text(output, "], al\n", 6);
    }
    if (backend_text(output, "QWORD PTR [rbp-", 15) == 0) {
        return 0;
    }
    if (backend_integer(output, scratch, scratch_capacity, slot) == 0) {
        return 0;
    }
    return backend_text(output, "], rax\n", 7);
}

int backend_address_local(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int slot
) {
    if (slot < 0) {
        output.error = 6;
        return 0;
    }
    if (backend_text(output, "  lea rax, [rbp-", 16) == 0) {
        return 0;
    }
    if (backend_integer(output, scratch, scratch_capacity, slot) == 0) {
        return 0;
    }
    return backend_text(output, "]\n  push rax\n", 13);
}

int backend_field_address(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int offset
) {
    if (offset < 0) {
        output.error = 6;
        return 0;
    }
    if (backend_text(output, "  pop rax\n", 10) == 0) {
        return 0;
    }
    if (offset == 0) {
        return backend_text(output, "  push rax\n", 11);
    }
    if (backend_text(output, "  add rax, ", 11) == 0) {
        return 0;
    }
    if (backend_integer(output, scratch, scratch_capacity, offset) == 0) {
        return 0;
    }
    return backend_text(output, "\n  push rax\n", 12);
}

int backend_load(
    struct BackendOutput* output,
    int type_kind
) {
    if (backend_text(output, "  pop rax\n  mov ", 16) == 0) {
        return 0;
    }
    if (type_kind == 2) {
        if (backend_text(output, "zx eax, BYTE PTR [rax]\n  push rax\n", 36) == 0) {
            return 0;
        }
    } else {
        if (backend_text(output, "rax, QWORD PTR [rax]\n  push rax\n", 32) == 0) {
            return 0;
        }
    }
    return 1;
}

int backend_scale(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int bytes
) {
    if (bytes < 0) {
        output.error = 6;
        return 0;
    }
    if (backend_text(output, "  pop rax\n  imul rax, ", 22) == 0) {
        return 0;
    }
    if (backend_integer(output, scratch, scratch_capacity, bytes) == 0) {
        return 0;
    }
    return backend_text(output, "\n  push rax\n", 12);
}

int backend_store(
    struct BackendOutput* output,
    int type_kind
) {
    if (backend_text(output, "  pop rax\n  pop rdi\n  mov ", 26) == 0) {
        return 0;
    }
    if (type_kind == 2) {
        return backend_text(output, "BYTE PTR [rdi], al\n", 19);
    }
    return backend_text(output, "QWORD PTR [rdi], rax\n", 21);
}

int backend_unary_negate(struct BackendOutput* output) {
    return backend_text(output, "  pop rax\n  neg rax\n  push rax\n", 31);
}

int backend_binary(struct BackendOutput* output, int operator) {
    if (operator == 1) {
        return backend_text(
            output,
            "  pop rdi\n  pop rax\n  add rax, rdi\n  push rax\n",
            46
        );
    }
    if (operator == 2) {
        return backend_text(
            output,
            "  pop rdi\n  pop rax\n  sub rax, rdi\n  push rax\n",
            46
        );
    }
    if (operator == 3) {
        return backend_text(
            output,
            "  pop rdi\n  pop rax\n  imul rax, rdi\n  push rax\n",
            47
        );
    }
    if (operator == 4) {
        return backend_text(
            output,
            "  pop rdi\n  pop rax\n  cqo\n  idiv rdi\n  push rax\n",
            48
        );
    }
    if (operator >= 5) {
        if (backend_text(output, "  pop rdi\n", 10) == 0) {
            return 0;
        }
        if (backend_text(output, "  pop rax\n", 10) == 0) {
            return 0;
        }
        if (backend_text(output, "  cmp rax, rdi\n", 15) == 0) {
            return 0;
        }
        if (operator == 5) {
            if (backend_text(output, "  sete al\n", 10) == 0) {
                return 0;
            }
        } else {
            if (operator == 6) {
                if (backend_text(output, "  setne al\n", 11) == 0) {
                    return 0;
                }
            } else {
                if (operator == 7) {
                    if (backend_text(output, "  setl al\n", 10) == 0) {
                        return 0;
                    }
                } else {
                    if (operator == 8) {
                        if (backend_text(output, "  setle al\n", 11) == 0) {
                            return 0;
                        }
                    } else {
                        if (operator == 9) {
                            if (backend_text(output, "  setg al\n", 10) == 0) {
                                return 0;
                            }
                        } else {
                            if (operator == 10) {
                                if (backend_text(output, "  setge al\n", 11) == 0) {
                                    return 0;
                                }
                            } else {
                                output.error = 3;
                                return 0;
                            }
                        }
                    }
                }
            }
        }
        if (backend_text(output, "  movzx rax, al\n", 16) == 0) {
            return 0;
        }
        return backend_text(output, "  push rax\n", 11);
    }
    output.error = 3;
    return 0;
}

int backend_block_label(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int block
) {
    if (block <= 0) {
        return 1;
    }
    if (backend_text(output, ".Lblock_", 8) == 0) {
        return 0;
    }
    if (backend_integer(output, scratch, scratch_capacity, block) == 0) {
        return 0;
    }
    return backend_text(output, ":\n", 2);
}

int backend_branch(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int then_block,
    int else_block
) {
    if (then_block <= 0) {
        output.error = 7;
        return 0;
    }
    if (else_block <= 0) {
        output.error = 7;
        return 0;
    }
    if (backend_text(output, "  pop rax\n  test rax, rax\n  jne .Lblock_", 40) == 0) {
        return 0;
    }
    if (backend_integer(output, scratch, scratch_capacity, then_block) == 0) {
        return 0;
    }
    if (backend_text(output, "\n  jmp .Lblock_", 15) == 0) {
        return 0;
    }
    if (backend_integer(output, scratch, scratch_capacity, else_block) == 0) {
        return 0;
    }
    return backend_char(output, '\n');
}

int backend_jump(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int target
) {
    if (target <= 0) {
        output.error = 7;
        return 0;
    }
    if (backend_text(output, "  jmp .Lblock_", 14) == 0) {
        return 0;
    }
    if (backend_integer(output, scratch, scratch_capacity, target) == 0) {
        return 0;
    }
    return backend_char(output, '\n');
}

int backend_call(
    struct BackendOutput* output,
    char* name,
    int name_length,
    int argument_count
) {
    if (name_length < 0) {
        output.error = 2;
        return 0;
    }
    if (argument_count < 0) {
        output.error = 3;
        return 0;
    }
    if (argument_count > 6) {
        output.error = 3;
        return 0;
    }
    let index = argument_count;
    while (index > 0) {
        index = index - 1;
        if (index == 0) {
            if (backend_text(output, "  pop rdi\n", 10) == 0) {
                return 0;
            }
        } else {
            if (index == 1) {
                if (backend_text(output, "  pop rsi\n", 10) == 0) {
                    return 0;
                }
            } else {
                if (index == 2) {
                    if (backend_text(output, "  pop rdx\n", 10) == 0) {
                        return 0;
                    }
                } else {
                    if (index == 3) {
                        if (backend_text(output, "  pop rcx\n", 10) == 0) {
                            return 0;
                        }
                    } else {
                        if (index == 4) {
                            if (backend_text(output, "  pop r8\n", 9) == 0) {
                                return 0;
                            }
                        } else {
                            if (backend_text(output, "  pop r9\n", 9) == 0) {
                                return 0;
                            }
                        }
                    }
                }
            }
        }
    }
    if (backend_text(output, "  call ", 7) == 0) {
        return 0;
    }
    if (backend_text(output, name, name_length) == 0) {
        return 0;
    }
    return backend_text(output, "\n  push rax\n", 12);
}

int backend_parameter_store(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int index
) {
    if (index < 0) {
        output.error = 3;
        return 0;
    }
    if (index > 5) {
        output.error = 3;
        return 0;
    }
    let slot = (index + 1) * 8;
    if (index == 0) {
        if (backend_text(output, "  mov rax, rdi\n  mov QWORD PTR [rbp-", 36) == 0) {
            return 0;
        }
    } else {
        if (index == 1) {
            if (backend_text(output, "  mov rax, rsi\n  mov QWORD PTR [rbp-", 36) == 0) {
                return 0;
            }
        } else {
            if (index == 2) {
                if (backend_text(output, "  mov rax, rdx\n  mov QWORD PTR [rbp-", 36) == 0) {
                    return 0;
                }
            } else {
                if (index == 3) {
                    if (backend_text(output, "  mov rax, rcx\n  mov QWORD PTR [rbp-", 36) == 0) {
                        return 0;
                    }
                } else {
                    if (index == 4) {
                        if (backend_text(output, "  mov rax, r8\n  mov QWORD PTR [rbp-", 35) == 0) {
                            return 0;
                        }
                    } else {
                        if (backend_text(output, "  mov rax, r9\n  mov QWORD PTR [rbp-", 35) == 0) {
                            return 0;
                        }
                    }
                }
            }
        }
    }
    if (backend_integer(output, scratch, scratch_capacity, slot) == 0) {
        return 0;
    }
    return backend_text(output, "], rax\n", 7);
}

int backend_constant(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    int value
) {
    if (backend_text(output, "  mov rax, ", 11) == 0) {
        return 0;
    }
    if (backend_integer(output, scratch, scratch_capacity, value) == 0) {
        return 0;
    }
    return backend_text(output, "\n  push rax\n", 12);
}

int backend_emit_instruction(
    struct BackendOutput* output,
    struct BackendInstruction* instruction
) {
    if (instruction.kind == 5) {
        if (instruction.operator == 1) {
            return backend_unary_negate(output);
        }
        output.error = 4;
        return 0;
    }
    if (instruction.kind == 6) {
        return backend_binary(output, instruction.operator);
    }
    if (instruction.kind == 11) {
        return backend_return(output, instruction.extra);
    }
    if (instruction.kind == 7) {
        return backend_call(
            output,
            instruction.name,
            instruction.name_length,
            instruction.extra
        );
    }
    output.error = 4;
    return 0;
}

int backend_emit_instruction_with_scratch(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    struct BackendInstruction* instruction
) {
    if (instruction.kind == 1) {
        return backend_constant(
            output,
            scratch,
            scratch_capacity,
            instruction.extra
        );
    }
    if (instruction.kind == 4) {
        return backend_load_local(
            output,
            scratch,
            scratch_capacity,
            instruction.extra,
            instruction.type_kind
        );
    }
    if (instruction.kind == 8) {
        return backend_store_local(
            output,
            scratch,
            scratch_capacity,
            instruction.extra,
            instruction.type_kind
        );
    }
    if (instruction.kind == 3) {
        return backend_address_local(
            output,
            scratch,
            scratch_capacity,
            instruction.extra
        );
    }
    if (instruction.kind == 9) {
        return backend_field_address(
            output,
            scratch,
            scratch_capacity,
            instruction.extra
        );
    }
    if (instruction.kind == 10) {
        return backend_load(output, instruction.type_kind);
    }
    if (instruction.kind == 14) {
        return backend_scale(
            output,
            scratch,
            scratch_capacity,
            instruction.extra
        );
    }
    if (instruction.kind == 15) {
        return backend_store(output, instruction.type_kind);
    }
    if (instruction.kind == 12) {
        return backend_branch(
            output,
            scratch,
            scratch_capacity,
            instruction.left,
            instruction.right
        );
    }
    if (instruction.kind == 13) {
        return backend_jump(
            output,
            scratch,
            scratch_capacity,
            instruction.left
        );
    }
    return backend_emit_instruction(output, instruction);
}

int backend_emit_block(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    struct BackendInstruction* instructions,
    int instruction_capacity,
    struct BackendBlock* block
) {
    if (block.start < 0) {
        output.error = 7;
        return 0;
    }
    if (block.count < 0) {
        output.error = 7;
        return 0;
    }
    if (block.start > instruction_capacity) {
        output.error = 7;
        return 0;
    }
    if (block.count > instruction_capacity - block.start) {
        output.error = 7;
        return 0;
    }
    let index = block.start;
    let end = block.start + block.count;
    while (index < end) {
        let instruction = &instructions[index];
        if (backend_emit_instruction_with_scratch(
            output,
            scratch,
            scratch_capacity,
            instruction
        ) == 0) {
            return 0;
        }
        index = index + 1;
    }
    return 1;
}

int backend_emit_function(
    struct BackendOutput* output,
    struct BackendProgram* program,
    char* name,
    int name_length,
    struct BackendFunction* function
) {
    if (name_length < 0) {
        output.error = 2;
        return 0;
    }
    if (function.start < 0) {
        output.error = 7;
        return 0;
    }
    if (function.count < 0) {
        output.error = 7;
        return 0;
    }
    if (function.start > program.instruction_capacity) {
        output.error = 7;
        return 0;
    }
    if (function.count > program.instruction_capacity - function.start) {
        output.error = 7;
        return 0;
    }
    if (function.block_start < 0) {
        output.error = 7;
        return 0;
    }
    if (function.block_count < 0) {
        output.error = 7;
        return 0;
    }
    if (function.block_start > program.block_capacity) {
        output.error = 7;
        return 0;
    }
    if (function.block_count > program.block_capacity - function.block_start) {
        output.error = 7;
        return 0;
    }
    if (backend_function_label(output, name, name_length) == 0) {
        return 0;
    }
    if (backend_function_prologue(output) == 0) {
        return 0;
    }
    if (function.parameter_count < 0) {
        output.error = 7;
        return 0;
    }
    if (function.parameter_count > 6) {
        output.error = 3;
        return 0;
    }
    if (function.parameter_count > 0) {
        if (backend_text(output, "  sub rsp, ", 11) == 0) {
            return 0;
        }
        if (backend_integer(
            output,
            program.scratch,
            program.scratch_capacity,
            function.parameter_count * 8
        ) == 0) {
            return 0;
        }
        if (backend_char(output, '\n') == 0) {
            return 0;
        }
    }
    let parameter_index = 0;
    while (parameter_index < function.parameter_count) {
        if (backend_parameter_store(
            output,
            program.scratch,
            program.scratch_capacity,
            parameter_index
        ) == 0) {
            return 0;
        }
        parameter_index = parameter_index + 1;
    }
    let block_index = function.block_start;
    let block_end = function.block_start + function.block_count;
    let instruction_end = function.start + function.count;
    while (block_index < block_end) {
        if (program.blocks[block_index].start < function.start) {
            output.error = 7;
            return 0;
        }
        if (program.blocks[block_index].start +
            program.blocks[block_index].count > instruction_end) {
            output.error = 7;
            return 0;
        }
        if (backend_block_label(
            output,
            program.scratch,
            program.scratch_capacity,
            program.blocks[block_index].id
        ) == 0) {
            return 0;
        }
        if (backend_emit_block(
            output,
            program.scratch,
            program.scratch_capacity,
            program.instructions,
            program.instruction_capacity,
            &program.blocks[block_index]
        ) == 0) {
            return 0;
        }
        block_index = block_index + 1;
    }
    return 1;
}
