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
    output.error = 3;
    return 0;
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
    output.error = 4;
    return 0;
}

int backend_emit_block(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    struct BackendInstruction* instructions,
    int instruction_capacity,
    struct BackendBlock* block
) {
    if (block.start < 0 || block.count < 0) {
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
        if (instruction.kind == 4) {
            if (backend_load_local(
                output,
                scratch,
                scratch_capacity,
                instruction.extra,
                instruction.type_kind
            ) == 0) {
                return 0;
            }
        } else {
            if (instruction.kind == 8) {
                if (backend_store_local(
                    output,
                    scratch,
                    scratch_capacity,
                    instruction.extra,
                    instruction.type_kind
                ) == 0) {
                    return 0;
                }
            } else {
                if (backend_emit_instruction(output, instruction) == 0) {
                    return 0;
                }
            }
        }
        index = index + 1;
    }
    return 1;
}

int backend_emit_function(
    struct BackendOutput* output,
    char* scratch,
    int scratch_capacity,
    char* name,
    int name_length,
    struct BackendInstruction* instructions,
    int instruction_capacity,
    struct BackendBlock* blocks,
    int block_capacity,
    struct BackendFunction* function
) {
    if (name_length < 0) {
        output.error = 2;
        return 0;
    }
    if (function.start < 0 || function.count < 0) {
        output.error = 7;
        return 0;
    }
    if (function.start > instruction_capacity) {
        output.error = 7;
        return 0;
    }
    if (function.count > instruction_capacity - function.start) {
        output.error = 7;
        return 0;
    }
    if (function.block_start < 0 || function.block_count < 0) {
        output.error = 7;
        return 0;
    }
    if (function.block_start > block_capacity) {
        output.error = 7;
        return 0;
    }
    if (function.block_count > block_capacity - function.block_start) {
        output.error = 7;
        return 0;
    }
    if (backend_function_label(output, name, name_length) == 0) {
        return 0;
    }
    if (backend_function_prologue(output) == 0) {
        return 0;
    }
    let block_index = function.block_start;
    let block_end = function.block_start + function.block_count;
    while (block_index < block_end) {
        if (backend_emit_block(
            output,
            scratch,
            scratch_capacity,
            instructions,
            instruction_capacity,
            &blocks[block_index]
        ) == 0) {
            return 0;
        }
        block_index = block_index + 1;
    }
    return 1;
}
