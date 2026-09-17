// Caller-owned K compiler pipeline.

struct CompilerContext {
    char* source;
    int source_length;
    struct Parser parser;
    struct AstProgram program;
    struct AstStorage ast;
    struct DeclarationStorage declarations;
    struct SemanticChecker checker;
    struct IrStorage ir;
}

struct CompilerBuffers {
    struct AstFunction* functions;
    int function_capacity;
    struct AstParameter* parameters;
    int parameter_capacity;
    struct AstExpression* expressions;
    int expression_capacity;
    struct AstArgument* arguments;
    int argument_capacity;
    struct AstStatement* statements;
    int statement_capacity;
    struct AstStruct* structs;
    int struct_capacity;
    struct AstField* fields;
    int field_capacity;
    struct SemanticFunction* semantic_functions;
    int semantic_function_capacity;
    struct SemanticStruct* semantic_structs;
    int semantic_struct_capacity;
    struct SemanticField* semantic_fields;
    int semantic_field_capacity;
    struct SemanticSymbol* symbols;
    int symbol_capacity;
    struct IrInstruction* instructions;
    int instruction_capacity;
    struct IrFunction* ir_functions;
    int ir_function_capacity;
    struct IrBlock* blocks;
    int block_capacity;
    struct BackendInstruction* backend_instructions;
    int backend_instruction_capacity;
    struct BackendBlock* backend_blocks;
    int backend_block_capacity;
    char* backend_scratch;
    int backend_scratch_capacity;
}

int compiler_init(
    struct CompilerContext* compiler,
    char* source,
    int source_length,
    struct CompilerBuffers* buffers
) {
    compiler.source = source;
    compiler.source_length = source_length;
    compiler.ast.functions = buffers.functions;
    compiler.ast.function_capacity = buffers.function_capacity;
    compiler.ast.parameters = buffers.parameters;
    compiler.ast.parameter_capacity = buffers.parameter_capacity;
    compiler.ast.expressions = buffers.expressions;
    compiler.ast.expression_capacity = buffers.expression_capacity;
    compiler.ast.arguments = buffers.arguments;
    compiler.ast.argument_capacity = buffers.argument_capacity;
    compiler.ast.statements = buffers.statements;
    compiler.ast.statement_capacity = buffers.statement_capacity;
    compiler.declarations.struct_records = buffers.structs;
    compiler.declarations.struct_record_capacity = buffers.struct_capacity;
    compiler.declarations.field_records = buffers.fields;
    compiler.declarations.field_record_capacity = buffers.field_capacity;
    compiler.declarations.semantic_functions = buffers.semantic_functions;
    compiler.declarations.semantic_function_capacity = buffers.semantic_function_capacity;
    compiler.declarations.semantic_records = buffers.semantic_structs;
    compiler.declarations.semantic_record_capacity = buffers.semantic_struct_capacity;
    compiler.declarations.semantic_members = buffers.semantic_fields;
    compiler.declarations.semantic_member_capacity = buffers.semantic_field_capacity;
    compiler.parser.declarations = &compiler.declarations;
    semantic_init(
        &compiler.checker,
        source,
        buffers.symbols,
        buffers.symbol_capacity
    );
    ir_init(
        &compiler.ir,
        buffers.instructions,
        buffers.instruction_capacity,
        buffers.ir_functions,
        buffers.ir_function_capacity
    );
    ir_block_storage(&compiler.ir, buffers.blocks, buffers.block_capacity);
    return 1;
}

int compiler_frontend(struct CompilerContext* compiler) {
    parser_init(
        &compiler.parser,
        compiler.source,
        compiler.source_length
    );
    parser_bind_declarations(&compiler.parser, &compiler.declarations);
    if (parse_program_tree(
        &compiler.parser,
        &compiler.program,
        &compiler.ast
    ) == 0) {
        return 0;
    }
    if (semantic_check_parser(
        &compiler.checker,
        &compiler.program,
        &compiler.parser,
        &compiler.ast
    ) == 0) {
        return 0;
    }
    if (ir_lower_program(
        &compiler.ir,
        &compiler.checker,
        &compiler.program,
        &compiler.ast
    ) == 0) {
        return 0;
    }
    if (ir_validate(&compiler.ir) == 0) {
        return 0;
    }
    return 1;
}

int compiler_emit_backend(
    struct CompilerContext* compiler,
    struct CompilerBuffers* buffers,
    char* output_data,
    int output_capacity
) {
    struct BackendOutput output;
    struct BackendProgram backend;
    if (backend_init(&output, output_data, output_capacity) == 0) {
        return 0;
    }
    if (backend_header(&output) == 0) {
        return 0;
    }
    if (compiler_frontend(compiler) == 0) {
        return 0;
    }
    if (compiler.ir.instruction_count > buffers.backend_instruction_capacity) {
        output.error = 7;
        return 0;
    }
    if (compiler.ir.block_count > buffers.backend_block_capacity) {
        output.error = 7;
        return 0;
    }
    let instruction_index = 0;
    while (instruction_index < compiler.ir.instruction_count) {
        buffers.backend_instructions[instruction_index].kind =
            compiler.ir.instructions[instruction_index].kind;
        buffers.backend_instructions[instruction_index].operator =
            compiler.ir.instructions[instruction_index].operator;
        buffers.backend_instructions[instruction_index].type_kind =
            compiler.ir.instructions[instruction_index].type_kind;
        buffers.backend_instructions[instruction_index].start =
            compiler.ir.instructions[instruction_index].start;
        buffers.backend_instructions[instruction_index].end =
            compiler.ir.instructions[instruction_index].end;
        buffers.backend_instructions[instruction_index].left =
            compiler.ir.instructions[instruction_index].left;
        buffers.backend_instructions[instruction_index].right =
            compiler.ir.instructions[instruction_index].right;
        buffers.backend_instructions[instruction_index].extra =
            compiler.ir.instructions[instruction_index].extra;
        buffers.backend_instructions[instruction_index].name =
            compiler.ir.instructions[instruction_index].name;
        buffers.backend_instructions[instruction_index].name_length =
            compiler.ir.instructions[instruction_index].name_length;
        instruction_index = instruction_index + 1;
    }
    let block_index = 0;
    while (block_index < compiler.ir.block_count) {
        buffers.backend_blocks[block_index].start =
            compiler.ir.blocks[block_index].start;
        buffers.backend_blocks[block_index].count =
            compiler.ir.blocks[block_index].count;
        buffers.backend_blocks[block_index].id =
            compiler.ir.blocks[block_index].id;
        block_index = block_index + 1;
    }
    backend.scratch = buffers.backend_scratch;
    backend.scratch_capacity = buffers.backend_scratch_capacity;
    backend.instructions = buffers.backend_instructions;
    backend.instruction_capacity = buffers.backend_instruction_capacity;
    backend.blocks = buffers.backend_blocks;
    backend.block_capacity = buffers.backend_block_capacity;
    let function_index = 0;
    while (function_index < compiler.ir.function_count) {
        struct BackendFunction function;
        function.start = compiler.ir.functions[function_index].start;
        function.count = compiler.ir.functions[function_index].count;
        function.return_type = compiler.ir.functions[function_index].return_type;
        function.parameter_start =
            compiler.ir.functions[function_index].parameter_start;
        function.parameter_count =
            compiler.ir.functions[function_index].parameter_count;
        function.block_start = compiler.ir.functions[function_index].block_start;
        function.block_count = compiler.ir.functions[function_index].block_count;
        let semantic_function = &compiler.checker.functions[function_index];
        if (backend_emit_function(
            &output,
            &backend,
            &semantic_function.source[semantic_function.name_start],
            semantic_function.name_end - semantic_function.name_start,
            &function
        ) == 0) {
            return 0;
        }
        function_index = function_index + 1;
    }
    return output.length;
}
