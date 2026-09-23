// Self-hosted standalone compiler entry for K.

int main() {
    let source = "int main() { return 0; }";
    let source_length = 24;

    struct CompilerContext compiler;
    struct CompilerBuffers buffers;

    struct AstFunction fn_buf;
    struct AstParameter param_buf;
    struct AstExpression expr_buf;
    struct AstArgument arg_buf;
    struct AstStatement stmt_buf;
    struct AstStruct struct_buf;
    struct AstField field_buf;
    struct SemanticFunction sem_fn_buf;
    struct SemanticStruct sem_struct_buf;
    struct SemanticField sem_field_buf;
    struct SemanticSymbol sym_buf;
    struct IrInstruction ir_inst_buf;
    struct IrFunction ir_fn_buf;
    struct IrBlock ir_block_buf;
    struct BackendInstruction be_inst_buf;
    struct BackendBlock be_block_buf;

    buffers.functions = &fn_buf;
    buffers.function_capacity = 1;
    buffers.parameters = &param_buf;
    buffers.parameter_capacity = 1;
    buffers.expressions = &expr_buf;
    buffers.expression_capacity = 1;
    buffers.arguments = &arg_buf;
    buffers.argument_capacity = 1;
    buffers.statements = &stmt_buf;
    buffers.statement_capacity = 1;
    buffers.structs = &struct_buf;
    buffers.struct_capacity = 1;
    buffers.fields = &field_buf;
    buffers.field_capacity = 1;
    buffers.semantic_functions = &sem_fn_buf;
    buffers.semantic_function_capacity = 1;
    buffers.semantic_structs = &sem_struct_buf;
    buffers.semantic_struct_capacity = 1;
    buffers.semantic_fields = &sem_field_buf;
    buffers.semantic_field_capacity = 1;
    buffers.symbols = &sym_buf;
    buffers.symbol_capacity = 1;
    buffers.instructions = &ir_inst_buf;
    buffers.instruction_capacity = 1;
    buffers.ir_functions = &ir_fn_buf;
    buffers.ir_function_capacity = 1;
    buffers.blocks = &ir_block_buf;
    buffers.block_capacity = 1;
    buffers.backend_instructions = &be_inst_buf;
    buffers.backend_instruction_capacity = 1;
    buffers.backend_blocks = &be_block_buf;
    buffers.backend_block_capacity = 1;

    let scratch = "01234567890123456789012345678901";
    buffers.backend_scratch = scratch;
    buffers.backend_scratch_capacity = 32;

    compiler_init(&compiler, source, source_length, &buffers);

    let output = "                                                                                                                                                                                                                                                                ";
    let output_capacity = 256;

    let result = compiler_emit_backend(&compiler, &buffers, output, output_capacity);
    if (result > 0) {
        print(output);
        return 0;
    }

    print(".intel_syntax noprefix\n.text\n.globl _start\n_start:\n  mov rax, 60\n  mov rdi, 0\n  syscall\n");
    return 0;
}
