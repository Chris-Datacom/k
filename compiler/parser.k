struct Token {
    int kind;
    int start;
    int end;
}

struct SourceSlice {
    char* source;
    int start;
    int end;
}

struct AstNode {
    int kind;
    int start;
    int end;
}

struct AstExpression {
    int kind;
    int operator;
    int start;
    int end;
    int left;
    int right;
    int extra;
    int extra_end;
}

struct AstArgument {
    int expression;
    int next;
}

struct ExpressionArena {
    struct AstExpression* nodes;
    int capacity;
    int count;
    int error;
    struct AstArgument* arguments;
    int argument_capacity;
    int argument_count;
}

struct AstFunction {
    int start;
    int end;
    int name_start;
    int name_end;
    int parameter_count;
    int parameter_start;
    int body_start;
    int body_count;
    int return_type;
}

struct AstParameter {
    int start;
    int end;
    int name_start;
    int name_end;
    int type_start;
    int type_end;
    int type_kind;
}

struct AstField {
    int start;
    int end;
    int name_start;
    int name_end;
    int type_start;
    int type_end;
    int type_kind;
}

struct AstStruct {
    int start;
    int end;
    int name_start;
    int name_end;
    int field_start;
    int field_count;
}

struct AstStatement {
    int kind;
    int start;
    int end;
    int left;
    int right;
    int extra;
    int body_start;
    int body_count;
    int else_start;
    int else_count;
}

struct StatementArena {
    struct AstStatement* nodes;
    int capacity;
    int count;
    int error;
}

struct AstBlock {
    int start;
    int end;
    int statement_start;
    int statement_count;
}

struct SemanticFunction {
    char* source;
    int name_start;
    int name_end;
    int return_type;
    int parameter_start;
    int parameter_count;
}

struct SemanticField {
    char* source;
    int name_start;
    int name_end;
    int type;
}

struct SemanticStruct {
    char* source;
    int name_start;
    int name_end;
    int field_start;
    int field_count;
}

struct AstStorage {
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
}

struct DeclarationStorage {
    struct AstStruct* struct_records;
    int struct_record_capacity;
    struct AstField* field_records;
    int field_record_capacity;
    struct SemanticFunction* semantic_functions;
    int semantic_function_capacity;
    struct SemanticStruct* semantic_records;
    int semantic_record_capacity;
    struct SemanticField* semantic_members;
    int semantic_member_capacity;
}

struct AstProgram {
    int start;
    int end;
    int function_count;
    int struct_count;
    int error;
}

struct SemanticSymbol {
    char* source;
    int start;
    int end;
    int scope;
    int kind;
    int type;
}

struct SemanticChecker {
    char* source;
    struct SemanticSymbol* symbols;
    int symbol_capacity;
    int symbol_count;
    struct ExpressionArena* expression_arena;
    struct StatementArena* statement_arena;
    int error;
    int scope;
    int return_type;
    struct SemanticFunction* functions;
    int function_capacity;
    int function_count;
    struct SemanticStruct* structs;
    int struct_capacity;
    int struct_count;
    struct SemanticField* fields;
    int field_capacity;
    int field_count;
    struct AstParameter* parameters;
}

struct Lexer {
    char* source;
    int length;
    int position;
}

struct Parser {
    struct Lexer lexer;
    struct Token current;
    int current_kind;
    int depth;
    int parameter_cursor;
    int field_cursor;
    struct DeclarationStorage* declarations;
    struct ExpressionArena expression_arena;
    struct StatementArena statement_arena;
}

int lex_one(char* source, int length, int position, struct Token* out) {
    let scanning = 1;
    while (scanning == 1) {
        if (position < length) {
            let skipped = source[position];
            if (skipped == ' ') {
                position = position + 1;
            } else {
                if (skipped == '\n') {
                    position = position + 1;
                } else {
                    if (skipped == '\t') {
                        position = position + 1;
                    } else {
                        if (skipped == '\r') {
                            position = position + 1;
                        } else {
                            if (skipped == '/') {
                                if (position + 1 < length) {
                                    if (source[position + 1] == '/') {
                                        position = position + 2;
                                        let comment_scanning = 1;
                                        while (comment_scanning == 1) {
                                            if (position < length) {
                                                if (source[position] != '\n') {
                                                    position = position + 1;
                                                } else {
                                                    comment_scanning = 0;
                                                    position = position + 1;
                                                }
                                            } else {
                                                comment_scanning = 0;
                                            }
                                        }
                                    } else {
                                        scanning = 0;
                                    }
                                } else {
                                    scanning = 0;
                                }
                            } else {
                                scanning = 0;
                            }
                        }
                    }
                }
            }
        } else {
            scanning = 0;
        }
    }

    if (position >= length) {
        out.kind = 0;
        out.start = position;
        out.end = position;
        return position;
    }

    let current = source[position];
    out.start = position;
    out.end = position + 1;

    if (current == '\'') {
        let next = position + 1;
        if (next < length) {
            if (source[next] == '\\') {
                next = next + 1;
                if (next < length) {
                    next = next + 1;
                }
            } else {
                next = next + 1;
            }
        }
        if (next < length) {
            if (source[next] == '\'') {
                out.kind = 14;
                out.end = next + 1;
                return next + 1;
            }
        }
        out.kind = -1;
        out.end = next;
        return next;
    }
    if (current == '"') {
        let next = position + 1;
        let closed = 0;
        let literal_scanning = 1;
        while (literal_scanning == 1) {
            if (next < length) {
                if (source[next] == '\\') {
                    next = next + 1;
                    if (next < length) {
                        next = next + 1;
                    }
                } else {
                    if (source[next] == '"') {
                        closed = 1;
                        next = next + 1;
                        literal_scanning = 0;
                    } else {
                        next = next + 1;
                    }
                }
            } else {
                literal_scanning = 0;
            }
        }
        if (closed == 1) {
            out.kind = 15;
            out.end = next;
            return next;
        }
        out.kind = -1;
        out.end = next;
        return next;
    }

    if (current >= '0') {
        if (current <= '9') {
            let next = position + 1;
            let number_scanning = 1;
            while (number_scanning == 1) {
                if (next < length) {
                    let digit = source[next];
                    if (digit >= '0') {
                        if (digit <= '9') {
                            next = next + 1;
                        } else {
                            number_scanning = 0;
                        }
                    } else {
                        number_scanning = 0;
                    }
                } else {
                    number_scanning = 0;
                }
            }
            out.kind = 2;
            out.end = next;
            return next;
        }
    }

    let identifier_start = 0;
    if (current == '_') {
        identifier_start = 1;
    }
    if (current >= 'a') {
        if (current <= 'z') {
            identifier_start = 1;
        }
    }
    if (current >= 'A') {
        if (current <= 'Z') {
            identifier_start = 1;
        }
    }
    if (identifier_start == 1) {
        let next = position + 1;
        let identifier_scanning = 1;
        while (identifier_scanning == 1) {
            if (next < length) {
                let character = source[next];
                let identifier_part = 0;
                if (character == '_') {
                    identifier_part = 1;
                }
                if (character >= 'a') {
                    if (character <= 'z') {
                        identifier_part = 1;
                    }
                }
                if (character >= 'A') {
                    if (character <= 'Z') {
                        identifier_part = 1;
                    }
                }
                if (character >= '0') {
                    if (character <= '9') {
                        identifier_part = 1;
                    }
                }
                if (identifier_part == 1) {
                    next = next + 1;
                } else {
                    identifier_scanning = 0;
                }
            } else {
                identifier_scanning = 0;
            }
        }
        out.kind = 1;
        out.end = next;
        if (next - position == 2) {
            if (source[position] == 'i') {
                if (source[position + 1] == 'f') {
                    out.kind = 4;
                }
            }
        }
        if (next - position == 3) {
            if (source[position] == 'i') {
                if (source[position + 1] == 'n') {
                    if (source[position + 2] == 't') {
                        out.kind = 5;
                    }
                }
            }
            if (source[position] == 'l') {
                if (source[position + 1] == 'e') {
                    if (source[position + 2] == 't') {
                        out.kind = 6;
                    }
                }
            }
        }
        if (next - position == 4) {
            if (source[position] == 'c') {
                if (source[position + 1] == 'h') {
                    if (source[position + 2] == 'a') {
                        if (source[position + 3] == 'r') {
                            out.kind = 7;
                        }
                    }
                }
            }
            if (source[position] == 'e') {
                if (source[position + 1] == 'l') {
                    if (source[position + 2] == 's') {
                        if (source[position + 3] == 'e') {
                            out.kind = 8;
                        }
                    }
                }
            }
            if (source[position] == 't') {
                if (source[position + 1] == 'r') {
                    if (source[position + 2] == 'u') {
                        if (source[position + 3] == 'e') {
                            out.kind = 9;
                        }
                    }
                }
            }
        }
        if (next - position == 5) {
            if (source[position] == 'w') {
                if (source[position + 1] == 'h') {
                    if (source[position + 2] == 'i') {
                        if (source[position + 3] == 'l') {
                            if (source[position + 4] == 'e') {
                                out.kind = 10;
                            }
                        }
                    }
                }
            }
            if (source[position] == 'f') {
                if (source[position + 1] == 'a') {
                    if (source[position + 2] == 'l') {
                        if (source[position + 3] == 's') {
                            if (source[position + 4] == 'e') {
                                out.kind = 11;
                            }
                        }
                    }
                }
            }
        }
        if (next - position == 6) {
            if (source[position] == 'r') {
                if (source[position + 1] == 'e') {
                    if (source[position + 2] == 't') {
                        if (source[position + 3] == 'u') {
                            if (source[position + 4] == 'r') {
                                if (source[position + 5] == 'n') {
                                    out.kind = 12;
                                }
                            }
                        }
                    }
                }
            }
            if (source[position] == 's') {
                if (source[position + 1] == 't') {
                    if (source[position + 2] == 'r') {
                        if (source[position + 3] == 'u') {
                            if (source[position + 4] == 'c') {
                                if (source[position + 5] == 't') {
                                    out.kind = 13;
                                }
                            }
                        }
                    }
                }
            }
        }
        return next;
    }

    if (current == '+') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '-') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '*') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '/') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '=') {
        out.kind = 3;
        if (position + 1 < length) {
            if (source[position + 1] == '=') {
                out.end = position + 2;
                return position + 2;
            }
        }
        return position + 1;
    }
    if (current == '!') {
        if (position + 1 < length) {
            if (source[position + 1] == '=') {
                out.kind = 3;
                out.end = position + 2;
                return position + 2;
            }
        }
        out.kind = -1;
        return position + 1;
    }
    if (current == '<') {
        out.kind = 3;
        if (position + 1 < length) {
            if (source[position + 1] == '=') {
                out.end = position + 2;
                return position + 2;
            }
        }
        return position + 1;
    }
    if (current == '>') {
        out.kind = 3;
        if (position + 1 < length) {
            if (source[position + 1] == '=') {
                out.end = position + 2;
                return position + 2;
            }
        }
        return position + 1;
    }
    if (current == '&') {
        out.kind = 3;
        return position + 1;
    }
    if (current == ';') {
        out.kind = 3;
        return position + 1;
    }
    if (current == ',') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '(') {
        out.kind = 3;
        return position + 1;
    }
    if (current == ')') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '{') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '}') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '[') {
        out.kind = 3;
        return position + 1;
    }
    if (current == ']') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '.') {
        out.kind = 3;
        return position + 1;
    }
    if (current == ';') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '(') {
        out.kind = 3;
        return position + 1;
    }
    if (current == ')') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '{') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '}') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '[') {
        out.kind = 3;
        return position + 1;
    }
    if (current == ']') {
        out.kind = 3;
        return position + 1;
    }
    if (current == ',') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '.') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '&') {
        out.kind = 3;
        return position + 1;
    }

    out.kind = -1;
    return position + 1;
}

void lexer_init(char* source, int length, struct Lexer* lexer) {
    lexer.source = source;
    lexer.length = length;
    lexer.position = 0;
    return;
}

void parser_bind_declarations(
    struct Parser* parser,
    struct DeclarationStorage* declarations
) {
    parser.declarations = declarations;
    return;
}

int next_token(struct Lexer* lexer, struct Token* out) {
    let next = lex_one(lexer.source, lexer.length, lexer.position, out);
    lexer.position = next;
    return out.kind;
}

void parser_init(struct Parser* parser, char* source, int length) {
    lexer_init(source, length, &parser.lexer);
    parser.current_kind = 0;
    parser.current.kind = 0;
    parser.current.start = 0;
    parser.current.end = 0;
    parser.depth = 0;
    parser.parameter_cursor = 0;
    parser.field_cursor = 0;
    return;
}

int parser_peek(struct Parser* parser) {
    if (parser.current_kind == 0) {
        parser.current_kind = next_token(&parser.lexer, &parser.current);
    }
    return parser.current_kind;
}

int parser_advance(struct Parser* parser) {
    let next = parser_peek(parser);
    parser.current_kind = 0;
    return next;
}

int parser_position(struct Parser* parser) {
    if (parser.current_kind != 0) {
        return parser.current.start;
    }
    return parser.lexer.position;
}

void token_slice(struct Parser* parser, struct SourceSlice* slice) {
    slice.source = parser.lexer.source;
    slice.start = parser.current.start;
    slice.end = parser.current.end;
    return;
}

int slice_length(struct SourceSlice* slice) {
    return slice.end - slice.start;
}

char slice_char(struct SourceSlice* slice, int offset) {
    return slice.source[slice.start + offset];
}

char parser_char(struct Parser* parser, int offset) {
    return parser.lexer.source[parser.current.start + offset];
}

int parser_at_char(struct Parser* parser, char value) {
    if (parser_peek(parser) != 3) {
        return 0;
    }
    if (parser_char(parser, 0) == value) {
        return 1;
    }
    return 0;
}

int parser_take_char(struct Parser* parser, char value) {
    if (parser_at_char(parser, value) == 1) {
        parser_advance(parser);
        return 1;
    }
    return 0;
}

int parser_take_kind(struct Parser* parser, int kind) {
    if (parser_peek(parser) == kind) {
        parser_advance(parser);
        return 1;
    }
    return 0;
}

void expression_arena_init(
    struct ExpressionArena* arena,
    struct AstExpression* nodes,
    int capacity,
    struct AstArgument* arguments,
    int argument_capacity
) {
    arena.nodes = nodes;
    arena.capacity = capacity;
    arena.count = 0;
    arena.error = 0;
    arena.arguments = arguments;
    arena.argument_capacity = argument_capacity;
    arena.argument_count = 0;
    return;
}

int expression_argument_add(
    struct ExpressionArena* arena,
    int expression,
    int next
) {
    if (arena.argument_count >= arena.argument_capacity) {
        arena.error = 1;
        return -1;
    }
    let index = arena.argument_count;
    arena.arguments[index].expression = expression;
    arena.arguments[index].next = next;
    arena.argument_count = arena.argument_count + 1;
    return index;
}

int expression_arena_add(
    struct ExpressionArena* arena,
    int kind,
    int operator,
    int start,
    int end,
    int left
) {
    if (arena.count >= arena.capacity) {
        arena.error = 1;
        return -1;
    }
    let index = arena.count;
    arena.nodes[index].kind = kind;
    arena.nodes[index].operator = operator;
    arena.nodes[index].start = start;
    arena.nodes[index].end = end;
    arena.nodes[index].left = left;
    arena.nodes[index].right = -1;
    arena.nodes[index].extra = 0;
    arena.nodes[index].extra_end = 0;
    arena.count = arena.count + 1;
    return index;
}

int expression_primary(
    struct Parser* parser,
    struct ExpressionArena* arena
) {
    let token_kind = parser_peek(parser);
    let start = parser.current.start;
    let end = parser.current.end;
    if (token_kind == 1) {
        parser_advance(parser);
        return expression_arena_add(arena, 1, 0, start, end, -1);
    }
    if (token_kind == 2) {
        parser_advance(parser);
        return expression_arena_add(arena, 2, 0, start, end, -1);
    }
    if (token_kind == 9) {
        parser_advance(parser);
        let boolean = expression_arena_add(arena, 3, 0, start, end, -1);
        if (boolean >= 0) {
            arena.nodes[boolean].extra = 1;
        }
        return boolean;
    }
    if (token_kind == 11) {
        parser_advance(parser);
        return expression_arena_add(arena, 3, 0, start, end, -1);
    }
    if (parser_take_char(parser, '(') == 1) {
        let nested = expression_tree(parser, 0, arena);
        if (nested < 0) {
            return -1;
        }
        if (parser_take_char(parser, ')') == 0) {
            arena.error = 2;
            return -1;
        }
        return nested;
    }
    arena.error = 2;
    return -1;
}

int expression_postfix(
    struct Parser* parser,
    struct ExpressionArena* arena,
    int expression
) {
    let running = 1;
    while (running == 1) {
        if (parser_take_char(parser, '[') == 1) {
            let index = expression_tree(parser, 0, arena);
            if (index < 0) {
                return -1;
            }
            let end = parser.current.start;
            if (parser_take_char(parser, ']') == 0) {
                arena.error = 2;
                return -1;
            }
            end = parser_position(parser);
            let index_node = expression_arena_add(arena, 7, 0, arena.nodes[expression].start, end, expression);
            if (index_node >= 0) {
                arena.nodes[index_node].right = index;
            }
            expression = index_node;
        } else {
            if (parser_take_char(parser, '.') == 1) {
                if (parser_peek(parser) != 1) {
                    arena.error = 2;
                    return -1;
                }
                let field = parser.current.start;
                let end = parser.current.end;
                parser_advance(parser);
                let field_node = expression_arena_add(arena, 8, 0, arena.nodes[expression].start, end, expression);
                if (field_node >= 0) {
                    arena.nodes[field_node].extra = field;
                    arena.nodes[field_node].extra_end = end;
                }
                expression = field_node;
            } else {
                if (parser_take_char(parser, '(') == 1) {
                    let first = -1;
                    let first_argument = -1;
                    let last = -1;
                    let arguments = 0;
                    if (parser_at_char(parser, ')') == 0) {
                        first = expression_tree(parser, 0, arena);
                        if (first < 0) {
                            return -1;
                        }
                        last = expression_argument_add(arena, first, -1);
                        if (last < 0) {
                            return -1;
                        }
                        first_argument = last;
                        arguments = 1;
                        while (parser_take_char(parser, ',') == 1) {
                            let argument = expression_tree(parser, 0, arena);
                            if (argument < 0) {
                                return -1;
                            }
                            let next = expression_argument_add(arena, argument, -1);
                            if (next < 0) {
                                return -1;
                            }
                            arena.arguments[last].next = next;
                            last = next;
                            arguments = arguments + 1;
                        }
                    }
                    let end = parser_position(parser);
                    if (parser_take_char(parser, ')') == 0) {
                        arena.error = 2;
                        return -1;
                    }
                    end = parser_position(parser);
                    let call_node = expression_arena_add(arena, 6, 0, arena.nodes[expression].start, end, expression);
                    if (call_node >= 0) {
                        arena.nodes[call_node].right = first;
                        arena.nodes[call_node].extra = arguments;
                        arena.nodes[call_node].extra_end = first_argument;
                    }
                    expression = call_node;
                } else {
                    running = 0;
                }
            }
        }
        if (expression < 0) {
            return -1;
        }
    }
    return expression;
}

int expression_prefix(
    struct Parser* parser,
    struct ExpressionArena* arena
) {
    if (parser_at_char(parser, '-') == 1) {
        let start = parser.current.start;
        parser_advance(parser);
        let operand = expression_prefix(parser, arena);
        if (operand < 0) {
            return -1;
        }
        return expression_arena_add(arena, 4, 1, start, arena.nodes[operand].end, operand);
    }
    if (parser_at_char(parser, '&') == 1) {
        let start = parser.current.start;
        parser_advance(parser);
        let operand = expression_prefix(parser, arena);
        if (operand < 0) {
            return -1;
        }
        return expression_arena_add(arena, 4, 2, start, arena.nodes[operand].end, operand);
    }
    if (parser_at_char(parser, '*') == 1) {
        let start = parser.current.start;
        parser_advance(parser);
        let operand = expression_prefix(parser, arena);
        if (operand < 0) {
            return -1;
        }
        return expression_arena_add(arena, 4, 3, start, arena.nodes[operand].end, operand);
    }
    let primary = expression_primary(parser, arena);
    if (primary < 0) {
        return -1;
    }
    return expression_postfix(parser, arena, primary);
}

int expression_precedence(struct Parser* parser) {
    if (parser_at_char(parser, '+') == 1) {
        return 2;
    }
    if (parser_at_char(parser, '-') == 1) {
        return 2;
    }
    if (parser_at_char(parser, '*') == 1) {
        return 3;
    }
    if (parser_at_char(parser, '/') == 1) {
        return 3;
    }
    return 0;
}

int expression_operator(struct Parser* parser) {
    if (parser_at_char(parser, '+') == 1) {
        return 1;
    }
    if (parser_at_char(parser, '-') == 1) {
        return 2;
    }
    if (parser_at_char(parser, '*') == 1) {
        return 3;
    }
    if (parser_at_char(parser, '/') == 1) {
        return 4;
    }
    return 0;
}

int expression_tree(
    struct Parser* parser,
    int minimum_precedence,
    struct ExpressionArena* arena
) {
    let left = expression_prefix(parser, arena);
    if (left < 0) {
        return -1;
    }
    let precedence = expression_precedence(parser);
    while (precedence >= minimum_precedence) {
        let operator = expression_operator(parser);
        parser_advance(parser);
        let right = expression_tree(parser, precedence + 1, arena);
        if (right < 0) {
            return -1;
        }
        let start = arena.nodes[left].start;
        let end = arena.nodes[right].end;
        let binary = expression_arena_add(arena, 5, operator, start, end, left);
        if (binary >= 0) {
            arena.nodes[binary].right = right;
        }
        left = binary;
        if (left < 0) {
            return -1;
        }
        precedence = expression_precedence(parser);
    }
    return left;
}

void statement_arena_init(
    struct StatementArena* arena,
    struct AstStatement* nodes,
    int capacity
) {
    arena.nodes = nodes;
    arena.capacity = capacity;
    arena.count = 0;
    arena.error = 0;
    return;
}

int statement_arena_add(
    struct StatementArena* arena,
    int kind,
    int start,
    int end,
    int left,
    int right
) {
    if (arena.count >= arena.capacity) {
        arena.error = 1;
        return -1;
    }
    let index = arena.count;
    arena.nodes[index].kind = kind;
    arena.nodes[index].start = start;
    arena.nodes[index].end = end;
    arena.nodes[index].left = left;
    arena.nodes[index].right = right;
    arena.nodes[index].extra = 0;
    arena.nodes[index].body_start = -1;
    arena.nodes[index].body_count = 0;
    arena.nodes[index].else_start = -1;
    arena.nodes[index].else_count = 0;
    arena.count = arena.count + 1;
    return index;
}

int parse_block_tree(
    struct Parser* parser,
    struct ExpressionArena* expressions,
    struct StatementArena* statements,
    int* block_start,
    int* block_count,
    int* block_end
) {
    if (parser_take_char(parser, '{') == 0) {
        statements.error = 2;
        return 0;
    }
    *block_start = parser_position(parser) - 1;
    let statement_start = statements.count;
    let running = 1;
    while (running == 1) {
        if (parser_take_char(parser, '}') == 1) {
            running = 0;
        } else {
            if (parser_peek(parser) == 0) {
                statements.error = 2;
                return 0;
            }
            if (parse_statement_tree(parser, expressions, statements) == 0) {
                return 0;
            }
        }
    }
    *block_end = parser_position(parser);
    *block_count = statements.count - statement_start;
    return 1;
}

int parse_statement_tree(
    struct Parser* parser,
    struct ExpressionArena* expressions,
    struct StatementArena* statements
) {
    let kind = parser_peek(parser);
    let start = parser.current.start;
    if (kind == 6) {
        parser_advance(parser);
        if (parser_take_kind(parser, 1) == 0) {
            statements.error = 2;
            return 0;
        }
        let name_start = parser.current.start;
        let name_end = parser.current.end;
        if (parser_take_char(parser, '=') == 0) {
            statements.error = 2;
            return 0;
        }
        let value = expression_tree(parser, 0, expressions);
        if (value < 0) {
            return 0;
        }
        if (parser_take_char(parser, ';') == 0) {
            statements.error = 2;
            return 0;
        }
        let index = statement_arena_add(statements, 1, start, parser_position(parser), name_start, value);
        if (index >= 0) {
            statements.nodes[index].extra = name_end;
        }
        return index + 1;
    }
    if (kind == 12) {
        parser_advance(parser);
        let value = -1;
        if (parser_at_char(parser, ';') == 0) {
            value = expression_tree(parser, 0, expressions);
            if (value < 0) {
                return 0;
            }
        }
        if (parser_take_char(parser, ';') == 0) {
            statements.error = 2;
            return 0;
        }
        let index = statement_arena_add(statements, 2, start, parser_position(parser), value, -1);
        return index + 1;
    }
    let control = 0;
    if (kind == 4) {
        control = 1;
    }
    if (kind == 10) {
        control = 1;
    }
    if (control == 1) {
        parser_advance(parser);
        if (parser_take_char(parser, '(') == 0) {
            statements.error = 2;
            return 0;
        }
        let condition = expression_tree(parser, 0, expressions);
        if (condition < 0) {
            statements.error = 2;
            return 0;
        }
        if (parser_take_char(parser, ')') == 0) {
            statements.error = 2;
            return 0;
        }
        let body_start = statements.count;
        let body = 0;
        let body_end = 0;
        if (parse_block_tree(parser, expressions, statements, &body_start, &body, &body_end) == 0) {
            return 0;
        }
        let statement_kind = 7;
        if (kind == 4) {
            statement_kind = 6;
        }
        let index = statement_arena_add(statements, statement_kind, start, body_end, condition, -1);
        if (index < 0) {
            return 0;
        }
        statements.nodes[index].body_start = body_start;
        statements.nodes[index].body_count = body;
        let has_else = 0;
        if (kind == 4) {
            if (parser_take_kind(parser, 8) == 1) {
                has_else = 1;
            }
        }
        if (has_else == 1) {
            let else_block_start = statements.count;
            let else_block_count = 0;
            let else_end = 0;
            if (parse_block_tree(parser, expressions, statements, &else_block_start, &else_block_count, &else_end) == 0) {
                return 0;
            }
            statements.nodes[index].else_start = else_block_start;
            statements.nodes[index].else_count = else_block_count;
            statements.nodes[index].end = else_end;
        }
        return index + 1;
    }
    if (parser_at_char(parser, '{') == 1) {
        let nested_start = 0;
        let nested_count = 0;
        let nested_end = 0;
        if (parse_block_tree(parser, expressions, statements, &nested_start, &nested_count, &nested_end) == 0) {
            return 0;
        }
        let index = statement_arena_add(statements, 5, nested_start, nested_end, -1, -1);
        if (index < 0) {
            return 0;
        }
        statements.nodes[index].body_start = nested_start;
        statements.nodes[index].body_count = nested_count;
        return index + 1;
    }
    let left = expression_tree(parser, 0, expressions);
    if (left < 0) {
        return 0;
    }
    if (parser_take_char(parser, '=') == 1) {
        let right = expression_tree(parser, 0, expressions);
        if (right < 0) {
            return 0;
        }
        if (parser_take_char(parser, ';') == 0) {
            statements.error = 2;
            return 0;
        }
        let index = statement_arena_add(statements, 3, start, parser_position(parser), left, right);
        return index + 1;
    }
    if (parser_take_char(parser, ';') == 0) {
        statements.error = 2;
        return 0;
    }
    let index = statement_arena_add(statements, 4, start, parser_position(parser), left, -1);
    return index + 1;
}

int parse_function_tree(
    struct Parser* parser,
    struct ExpressionArena* expressions,
    struct StatementArena* statements,
    struct AstFunction* function,
    struct AstParameter* parameters,
    int parameter_capacity
) {
    let start = parser_position(parser);
    let return_type = parse_type_kind(parser);
    if (return_type == 0) {
        return 0;
    }
    if (parser_peek(parser) != 1) {
        return 0;
    }
    let name_start = parser.current.start;
    let name_end = parser.current.end;
    parser_advance(parser);
    if (parser_take_char(parser, '(') == 0) {
        return 0;
    }
    let parameter_count = 0;
    let parameter_start = parser.parameter_cursor;
    let running = 1;
    while (running == 1) {
        if (parser_take_char(parser, ')') == 1) {
            running = 0;
        } else {
            let type_start = parser_position(parser);
            let parameter_type = parse_type_kind(parser);
            if (parameter_type == 0) {
                return 0;
            }
            if (parser_peek(parser) != 1) {
                return 0;
            }
            let parameter = parser.parameter_cursor;
            if (parameter >= parameter_capacity) {
                statements.error = 3;
                return 0;
            }
            let parameter_name_start = parser.current.start;
            let parameter_name_end = parser.current.end;
            parser_advance(parser);
            parameters[parameter].start = type_start;
            parameters[parameter].end = parameter_name_end;
            parameters[parameter].name_start = parameter_name_start;
            parameters[parameter].name_end = parameter_name_end;
            parameters[parameter].type_start = type_start;
            parameters[parameter].type_end = parameter_name_start;
            parameters[parameter].type_kind = parameter_type;
            parameter_count = parameter_count + 1;
            parser.parameter_cursor = parser.parameter_cursor + 1;
            if (parser_take_char(parser, ',') == 0) {
                if (parser_take_char(parser, ')') == 0) {
                    return 0;
                }
                running = 0;
            }
        }
    }
    let body_end = 0;
    let body_statement_start = 0;
    let body_statement_count = 0;
    if (parse_block_tree(parser, expressions, statements, &body_statement_start, &body_statement_count, &body_end) == 0) {
        return 0;
    }
    function.start = start;
    function.end = body_end;
    function.name_start = name_start;
    function.name_end = name_end;
    function.parameter_count = parameter_count;
    function.parameter_start = parameter_start;
    function.body_start = body_statement_start;
    function.body_count = body_statement_count;
    function.return_type = return_type;
    return 1;
}

int parse_struct_tree(
    struct Parser* parser,
    struct AstStruct* definition,
    struct AstField* fields,
    int field_capacity
) {
    let start = parser_position(parser);
    if (parser_take_kind(parser, 13) == 0) {
        return 0;
    }
    if (parser_peek(parser) != 1) {
        return 0;
    }
    let name_start = parser.current.start;
    let name_end = parser.current.end;
    parser_advance(parser);
    if (parser_take_char(parser, '{') == 0) {
        return 0;
    }
    let field_start = parser.field_cursor;
    let field_count = 0;
    let running = 1;
    while (running == 1) {
        if (parser_take_char(parser, '}') == 1) {
            running = 0;
        } else {
            let field_begin = parser_position(parser);
            let field_type = parse_type_kind(parser);
            if (field_type == 0) {
                return 0;
            }
            if (parser_peek(parser) != 1) {
                return 0;
            }
            let field = parser.field_cursor;
            if (field >= field_capacity) {
                return 0;
            }
            let field_name_start = parser.current.start;
            let field_name_end = parser.current.end;
            parser_advance(parser);
            if (parser_take_char(parser, ';') == 0) {
                return 0;
            }
            fields[field].start = field_begin;
            fields[field].end = parser_position(parser);
            fields[field].name_start = field_name_start;
            fields[field].name_end = field_name_end;
            fields[field].type_start = field_begin;
            fields[field].type_end = field_name_start;
            fields[field].type_kind = field_type;
            parser.field_cursor = parser.field_cursor + 1;
            field_count = field_count + 1;
        }
    }
    definition.start = start;
    definition.end = parser_position(parser);
    definition.name_start = name_start;
    definition.name_end = name_end;
    definition.field_start = field_start;
    definition.field_count = field_count;
    return 1;
}

int parser_is_void(struct Parser* parser) {
    if (parser_peek(parser) != 1) {
        return 0;
    }
    if (parser.current.end - parser.current.start != 4) {
        return 0;
    }
    if (parser_char(parser, 0) == 'v') {
        if (parser_char(parser, 1) == 'o') {
            if (parser_char(parser, 2) == 'i') {
                if (parser_char(parser, 3) == 'd') {
                    return 1;
                }
            }
        }
    }
    return 0;
}

int parse_type_kind(struct Parser* parser) {
    let parsed = 0;
    let type = 0;
    if (parser_take_kind(parser, 5) == 1) {
        parsed = 1;
        type = 1;
    } else {
        if (parser_take_kind(parser, 7) == 1) {
            parsed = 1;
            type = 2;
        } else {
            if (parser_is_void(parser) == 1) {
                parser_advance(parser);
                parsed = 1;
                type = 3;
            } else {
                if (parser_take_kind(parser, 13) == 1) {
                    if (parser_take_kind(parser, 1) == 1) {
                        parsed = 1;
                        type = 5;
                    }
                }
            }
        }
    }
    while (parser_take_char(parser, '*') == 1) {
        if (type == 1) {
            type = 4;
        } else {
            if (type == 2) {
                type = 4;
            } else {
                if (type == 5) {
                    type = 4;
                } else {
                    if (type == 3) {
                        type = 4;
                    }
                }
            }
        }
    }
    if (parsed == 0) {
        return 0;
    }
    return type;
}

int parse_type(struct Parser* parser) {
    if (parse_type_kind(parser) == 0) {
        return 0;
    }
    return 1;
}

int parse_expression(struct Parser* parser, char terminator) {
    let parsed = 0;
    let parens = 0;
    let brackets = 0;
    let running = 1;
    while (running == 1) {
        let kind = parser_peek(parser);
        if (kind == 0) {
            running = 0;
        } else {
            if (kind == 1) {
                parser_advance(parser);
                parsed = 1;
            } else {
                if (kind == 2) {
                    parser_advance(parser);
                    parsed = 1;
                } else {
                    if (kind == 3) {
                        if (parser_at_char(parser, terminator) == 1) {
                            if (parens == 0) {
                                if (brackets == 0) {
                                    running = 0;
                                } else {
                                    parser_advance(parser);
                                }
                            } else {
                                parser_advance(parser);
                            }
                        } else {
                            if (parser_at_char(parser, '(') == 1) {
                                parens = parens + 1;
                                parser_advance(parser);
                            } else {
                                if (parser_at_char(parser, ')') == 1) {
                                    if (parens > 0) {
                                        parens = parens - 1;
                                        parser_advance(parser);
                                    } else {
                                        running = 0;
                                    }
                                } else {
                                    if (parser_at_char(parser, '[') == 1) {
                                        brackets = brackets + 1;
                                        parser_advance(parser);
                                    } else {
                                        if (parser_at_char(parser, ']') == 1) {
                                            if (brackets > 0) {
                                                brackets = brackets - 1;
                                                parser_advance(parser);
                                            } else {
                                                running = 0;
                                            }
                                        } else {
                                            parser_advance(parser);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        running = 0;
                    }
                }
            }
        }
    }
    return parsed;
}

int parse_block(struct Parser* parser) {
    if (parser_take_char(parser, '{') == 0) {
        return 0;
    }
    let statements = 0;
    let running = 1;
    while (running == 1) {
        if (parser_take_char(parser, '}') == 1) {
            running = 0;
        } else {
            if (parser_peek(parser) == 0) {
                running = 0;
            } else {
                if (parse_statement(parser) == 0) {
                    running = 0;
                } else {
                    statements = statements + 1;
                }
            }
        }
    }
    return statements + 1;
}

int parse_statement(struct Parser* parser) {
    let kind = parser_peek(parser);
    if (kind == 6) {
        parser_advance(parser);
        if (parser_take_kind(parser, 1) == 0) {
            return 0;
        }
        if (parser_take_char(parser, '=') == 0) {
            return 0;
        }
        if (parse_expression(parser, ';') == 0) {
            return 0;
        }
        return parser_take_char(parser, ';');
    }
    if (kind == 12) {
        parser_advance(parser);
        if (parser_at_char(parser, ';') == 0) {
            if (parse_expression(parser, ';') == 0) {
                return 0;
            }
        }
        return parser_take_char(parser, ';');
    }
    if (kind == 4) {
        parser_advance(parser);
        if (parser_take_char(parser, '(') == 0) {
            return 0;
        }
        if (parse_expression(parser, ')') == 0) {
            return 0;
        }
        if (parser_take_char(parser, ')') == 0) {
            return 0;
        }
        return parse_block(parser);
    }
    if (kind == 10) {
        parser_advance(parser);
        if (parser_take_char(parser, '(') == 0) {
            return 0;
        }
        if (parse_expression(parser, ')') == 0) {
            return 0;
        }
        if (parser_take_char(parser, ')') == 0) {
            return 0;
        }
        return parse_block(parser);
    }
    if (parser_at_char(parser, '{') == 1) {
        return parse_block(parser);
    }
    if (parse_expression(parser, ';') == 0) {
        return 0;
    }
    return parser_take_char(parser, ';');
}

int parse_struct(struct Parser* parser) {
    if (parser_take_kind(parser, 13) == 0) {
        return 0;
    }
    if (parser_take_kind(parser, 1) == 0) {
        return 0;
    }
    if (parser_take_char(parser, '{') == 0) {
        return 0;
    }
    let running = 1;
    while (running == 1) {
        if (parser_take_char(parser, '}') == 1) {
            running = 0;
        } else {
            if (parse_type(parser) == 0) {
                return 0;
            }
            if (parser_take_kind(parser, 1) == 0) {
                return 0;
            }
            if (parser_take_char(parser, ';') == 0) {
                return 0;
            }
        }
    }
    return 1;
}

int parse_function(struct Parser* parser) {
    if (parse_type(parser) == 0) {
        return 0;
    }
    if (parser_take_kind(parser, 1) == 0) {
        return 0;
    }
    if (parser_take_char(parser, '(') == 0) {
        return 0;
    }
    let running = 1;
    while (running == 1) {
        if (parser_take_char(parser, ')') == 1) {
            running = 0;
        } else {
            if (parse_type(parser) == 0) {
                return 0;
            }
            if (parser_take_kind(parser, 1) == 0) {
                return 0;
            }
            if (parser_take_char(parser, ',') == 1) {
                parser.depth = parser.depth + 1;
            } else {
                if (parser_take_char(parser, ')') == 1) {
                    running = 0;
                } else {
                    return 0;
                }
            }
        }
    }
    return parse_block(parser);
}

int parse_expression_ast(struct Parser* parser, char terminator, struct AstNode* node) {
    let start = parser_position(parser);
    let parsed = parse_expression(parser, terminator);
    if (parsed == 0) {
        return 0;
    }
    node.kind = 1;
    node.start = start;
    node.end = parser_position(parser);
    return 1;
}

int parse_function_ast(struct Parser* parser, struct AstFunction* function) {
    let start = parser_position(parser);
    if (parse_type(parser) == 0) {
        return 0;
    }
    if (parser_peek(parser) != 1) {
        return 0;
    }
    let name_start = parser.current.start;
    let name_end = parser.current.end;
    parser_advance(parser);
    if (parser_take_char(parser, '(') == 0) {
        return 0;
    }
    let parameters = 0;
    let running = 1;
    while (running == 1) {
        if (parser_take_char(parser, ')') == 1) {
            running = 0;
        } else {
            if (parse_type(parser) == 0) {
                return 0;
            }
            if (parser_take_kind(parser, 1) == 0) {
                return 0;
            }
            parameters = parameters + 1;
            if (parser_take_char(parser, ',') == 0) {
                if (parser_take_char(parser, ')') == 0) {
                    return 0;
                }
                running = 0;
            }
        }
    }
    if (parse_block(parser) == 0) {
        return 0;
    }
    function.start = start;
    function.end = parser_position(parser);
    function.name_start = name_start;
    function.name_end = name_end;
    function.parameter_count = parameters;
    return 1;
}

int parse_program_ast(
    struct Parser* parser,
    struct AstProgram* program,
    struct AstFunction* functions,
    int function_capacity
) {
    let start = parser_position(parser);
    let function_count = 0;
    let struct_count = 0;
    let running = 1;
    program.error = 0;
    while (running == 1) {
        if (parser_peek(parser) == 0) {
            running = 0;
        } else {
            if (parser_peek(parser) == 13) {
                if (struct_count >= parser.declarations.struct_record_capacity) {
                    program.error = 2;
                    return 0;
                }
                if (parse_struct_tree(
                    parser,
                    &parser.declarations.struct_records[struct_count],
                    parser.declarations.field_records,
                    parser.declarations.field_record_capacity
                ) == 0) {
                    program.error = 1;
                    return 0;
                }
                struct_count = struct_count + 1;
            } else {
                if (function_count >= function_capacity) {
                    program.error = 2;
                    return 0;
                }
                if (parse_function_ast(parser, &functions[function_count]) == 0) {
                    program.error = 1;
                    return 0;
                }
                function_count = function_count + 1;
            }
        }
    }
    program.start = start;
    program.end = parser_position(parser);
    program.function_count = function_count;
    program.struct_count = struct_count;
    return function_count + struct_count;
}

int parse_program_tree(
    struct Parser* parser,
    struct AstProgram* program,
    struct AstStorage* storage
) {
    let start = parser_position(parser);
    let function_count = 0;
    let struct_count = 0;
    let running = 1;
    expression_arena_init(
        &parser.expression_arena,
        storage.expressions,
        storage.expression_capacity,
        storage.arguments,
        storage.argument_capacity
    );
    statement_arena_init(
        &parser.statement_arena,
        storage.statements,
        storage.statement_capacity
    );
    program.error = 0;
    while (running == 1) {
        if (parser_peek(parser) == 0) {
            running = 0;
        } else {
            if (parser_peek(parser) == 13) {
                if (parse_struct(parser) == 0) {
                    program.error = 1;
                    return 0;
                }
                struct_count = struct_count + 1;
            } else {
                if (function_count >= storage.function_capacity) {
                    program.error = 2;
                    return 0;
                }
                if (parse_function_tree(
                    parser,
                    &parser.expression_arena,
                    &parser.statement_arena,
                    &storage.functions[function_count],
                    storage.parameters,
                    storage.parameter_capacity
                ) == 0) {
                    program.error = 1;
                    return 0;
                }
                function_count = function_count + 1;
            }
        }
    }
    if (parser.expression_arena.error != 0) {
        program.error = 3;
        return 0;
    }
    if (parser.statement_arena.error != 0) {
        program.error = 3;
        return 0;
    }
    program.start = start;
    program.end = parser_position(parser);
    program.function_count = function_count;
    program.struct_count = struct_count;
    return function_count + struct_count;
}

void semantic_init(
    struct SemanticChecker* checker,
    char* source,
    struct SemanticSymbol* symbols,
    int symbol_capacity
) {
    checker.source = source;
    checker.symbols = symbols;
    checker.symbol_capacity = symbol_capacity;
    checker.symbol_count = 0;
    checker.error = 0;
    checker.scope = 0;
    checker.return_type = 3;
    checker.function_capacity = 0;
    checker.function_count = 0;
    checker.struct_capacity = 0;
    checker.struct_count = 0;
    checker.field_capacity = 0;
    checker.field_count = 0;
    return;
}

void semantic_bind(
    struct SemanticChecker* checker,
    struct ExpressionArena* expressions,
    struct StatementArena* statements
) {
    checker.expression_arena = expressions;
    checker.statement_arena = statements;
    return;
}

int semantic_same_name(
    struct SemanticChecker* checker,
    int left_start,
    int left_end,
    int right_start,
    int right_end
) {
    if (left_end - left_start != right_end - right_start) {
        return 0;
    }
    let offset = 0;
    let same = 1;
    while (offset < left_end - left_start) {
        if (checker.source[left_start + offset] != checker.source[right_start + offset]) {
            same = 0;
        }
        offset = offset + 1;
    }
    return same;
}

void semantic_bind_signatures(
    struct SemanticChecker* checker,
    struct DeclarationStorage* declarations,
    struct AstParameter* parameters
) {
    checker.functions = declarations.semantic_functions;
    checker.function_capacity = declarations.semantic_function_capacity;
    checker.structs = declarations.semantic_records;
    checker.struct_capacity = declarations.semantic_record_capacity;
    checker.fields = declarations.semantic_members;
    checker.field_capacity = declarations.semantic_member_capacity;
    checker.function_count = 0;
    checker.struct_count = 0;
    checker.field_count = 0;
    checker.parameters = parameters;
    return;
}

int semantic_collect_signatures(
    struct SemanticChecker* checker,
    struct AstProgram* program,
    struct AstStorage* storage,
    struct DeclarationStorage* declarations
) {
    let index = 0;
    while (index < program.function_count) {
        if (checker.function_count >= checker.function_capacity) {
            checker.error = 11;
            return 0;
        }
        let source_function = &storage.functions[index];
        let prior = 0;
        while (prior < checker.function_count) {
            if (semantic_same_name(
                checker,
                source_function.name_start,
                source_function.name_end,
                checker.functions[prior].name_start,
                checker.functions[prior].name_end
            ) == 1) {
                checker.error = 2;
                return 0;
            }
            prior = prior + 1;
        }
        checker.functions[index].source = checker.source;
        checker.functions[index].name_start = source_function.name_start;
        checker.functions[index].name_end = source_function.name_end;
        checker.functions[index].return_type = source_function.return_type;
        checker.functions[index].parameter_start = source_function.parameter_start;
        checker.functions[index].parameter_count = source_function.parameter_count;
        checker.function_count = checker.function_count + 1;
        index = index + 1;
    }
    index = 0;
    while (index < program.struct_count) {
        if (checker.struct_count >= checker.struct_capacity) {
            checker.error = 11;
            return 0;
        }
        let source_struct = &declarations.struct_records[index];
        checker.structs[index].source = checker.source;
        checker.structs[index].name_start = source_struct.name_start;
        checker.structs[index].name_end = source_struct.name_end;
        checker.structs[index].field_start = source_struct.field_start;
        checker.structs[index].field_count = source_struct.field_count;
        checker.struct_count = checker.struct_count + 1;
        let field_index = 0;
        while (field_index < source_struct.field_count) {
            if (checker.field_count >= checker.field_capacity) {
                checker.error = 11;
                return 0;
            }
            let source_field = &declarations.field_records[source_struct.field_start + field_index];
            checker.fields[checker.field_count].source = checker.source;
            checker.fields[checker.field_count].name_start = source_field.name_start;
            checker.fields[checker.field_count].name_end = source_field.name_end;
            checker.fields[checker.field_count].type = source_field.type_kind;
            checker.field_count = checker.field_count + 1;
            field_index = field_index + 1;
        }
        index = index + 1;
    }
    return 1;
}

int semantic_function_index(
    struct SemanticChecker* checker,
    int start,
    int end
) {
    let index = 0;
    while (index < checker.function_count) {
        if (semantic_same_name(
            checker,
            start,
            end,
            checker.functions[index].name_start,
            checker.functions[index].name_end
        ) == 1) {
            return index;
        }
        index = index + 1;
    }
    checker.error = 12;
    return -1;
}

int semantic_field_type(
    struct SemanticChecker* checker,
    int start,
    int end
) {
    let index = 0;
    while (index < checker.field_count) {
        if (semantic_same_name(
            checker,
            start,
            end,
            checker.fields[index].name_start,
            checker.fields[index].name_end
        ) == 1) {
            return checker.fields[index].type;
        }
        index = index + 1;
    }
    checker.error = 13;
    return 0;
}

int semantic_declare(
    struct SemanticChecker* checker,
    int start,
    int end,
    int kind,
    int type
) {
    let index = 0;
    while (index < checker.symbol_count) {
        if (checker.symbols[index].scope == checker.scope) {
            if (semantic_same_name(
                checker,
                start,
                end,
                checker.symbols[index].start,
                checker.symbols[index].end
            ) == 1) {
                checker.error = 2;
                return 0;
            }
        }
        index = index + 1;
    }
    if (checker.symbol_count >= checker.symbol_capacity) {
        checker.error = 1;
        return 0;
    }
    checker.symbols[checker.symbol_count].source = checker.source;
    checker.symbols[checker.symbol_count].start = start;
    checker.symbols[checker.symbol_count].end = end;
    checker.symbols[checker.symbol_count].scope = checker.scope;
    checker.symbols[checker.symbol_count].kind = kind;
    checker.symbols[checker.symbol_count].type = type;
    checker.symbol_count = checker.symbol_count + 1;
    return 1;
}

int semantic_resolve(
    struct SemanticChecker* checker,
    int start,
    int end
) {
    let index = checker.symbol_count - 1;
    while (index >= 0) {
        if (semantic_same_name(
            checker,
            start,
            end,
            checker.symbols[index].start,
            checker.symbols[index].end
        ) == 1) {
            return 1;
        }
        index = index - 1;
    }
    checker.error = 3;
    return 0;
}

int semantic_name_type(
    struct SemanticChecker* checker,
    int start,
    int end
) {
    let index = checker.symbol_count - 1;
    while (index >= 0) {
        if (semantic_same_name(
            checker,
            start,
            end,
            checker.symbols[index].start,
            checker.symbols[index].end
        ) == 1) {
            return checker.symbols[index].type;
        }
        index = index - 1;
    }
    checker.error = 3;
    return 0;
}

int semantic_expression_type(
    struct SemanticChecker* checker,
    struct ExpressionArena* expressions,
    int root
) {
    if (root < 0) {
        return 3;
    }
    if (root >= checker.expression_arena.count) {
        checker.error = 4;
        return 0;
    }
    let node = &expressions.nodes[root];
    if (node.kind == 1) {
        return semantic_name_type(checker, node.start, node.end);
    }
    if (node.kind == 2) {
        return 1;
    }
    if (node.kind == 3) {
        return 6;
    }
    if (node.kind == 4) {
        let operand = semantic_expression_type(checker, expressions, node.left);
        if (operand == 0) {
            return 0;
        }
        if (node.operator == 1) {
            if (operand != 1) {
                checker.error = 5;
                return 0;
            }
        }
        if (node.operator == 2) {
            return 4;
        }
        if (node.operator == 3) {
            if (operand != 4) {
                checker.error = 5;
                return 0;
            }
            return 1;
        }
        return operand;
    }
    if (node.kind == 5) {
        let left = semantic_expression_type(checker, expressions, node.left);
        let right = semantic_expression_type(checker, expressions, node.right);
        if (left == 0) {
            return 0;
        }
        if (right == 0) {
            return 0;
        }
        if (node.operator >= 5) {
            if (left != right) {
                checker.error = 6;
                return 0;
            }
            return 6;
        }
        if (left != 1) {
            checker.error = 5;
            return 0;
        }
        if (right != 1) {
            checker.error = 5;
            return 0;
        }
        return 1;
    }
    if (node.kind == 6) {
        if (node.left < 0) {
            checker.error = 4;
            return 0;
        }
        if (node.left >= checker.expression_arena.count) {
            checker.error = 4;
            return 0;
        }
        let callee = &expressions.nodes[node.left];
        if (callee.kind != 1) {
            checker.error = 7;
            return 0;
        }
        let function_index = semantic_function_index(checker, callee.start, callee.end);
        if (function_index < 0) {
            return 0;
        }
        let function = &checker.functions[function_index];
        if (node.extra != function.parameter_count) {
            checker.error = 14;
            return 0;
        }
        let argument_index = node.extra_end;
        let argument_position = 0;
        while (argument_position < node.extra) {
            if (argument_index < 0) {
                checker.error = 4;
                return 0;
            }
            if (argument_index >= expressions.argument_count) {
                checker.error = 4;
                return 0;
            }
            let argument = &expressions.arguments[argument_index];
            let argument_type = semantic_expression_type(checker, expressions, argument.expression);
            if (argument_type == 0) {
                return 0;
            }
            let parameter_index = function.parameter_start + argument_position;
            if (parameter_index < 0) {
                checker.error = 4;
                return 0;
            }
            let parameter_type = checker.parameters[parameter_index].type_kind;
            if (argument_type != parameter_type) {
                checker.error = 9;
                return 0;
            }
            argument_index = argument.next;
            argument_position = argument_position + 1;
        }
        if (argument_index >= 0) {
            checker.error = 4;
            return 0;
        }
        return function.return_type;
    }
    if (node.kind == 7) {
        let base = semantic_expression_type(checker, expressions, node.left);
        let index = semantic_expression_type(checker, expressions, node.right);
        if (base == 0) {
            return 0;
        }
        if (index == 0) {
            return 0;
        }
        if (base != 4) {
            checker.error = 5;
            return 0;
        }
        if (index != 1) {
            checker.error = 5;
            return 0;
        }
        return 1;
    }
    if (node.kind == 8) {
        let base = semantic_expression_type(checker, expressions, node.left);
        if (base == 0) {
            return 0;
        }
        if (base != 5) {
            checker.error = 13;
            return 0;
        }
        return semantic_field_type(checker, node.extra, node.extra_end);
    }
    checker.error = 4;
    return 0;
}

int semantic_expression(
    struct SemanticChecker* checker,
    struct ExpressionArena* expressions,
    int root
) {
    if (root < 0) {
        return 1;
    }
    if (root >= checker.expression_arena.count) {
        checker.error = 4;
        return 0;
    }
    let node = &expressions.nodes[root];
    if (node.kind == 1) {
        return semantic_resolve(checker, node.start, node.end);
    }
    if (node.kind == 2) {
        return 1;
    }
    if (node.kind == 3) {
        return 1;
    }
    if (node.kind == 4) {
        return semantic_expression(checker, expressions, node.left);
    }
    if (node.kind == 5) {
        if (semantic_expression(checker, expressions, node.left) == 0) {
            return 0;
        }
        return semantic_expression(checker, expressions, node.right);
    }
    if (node.kind == 6) {
        if (semantic_expression(checker, expressions, node.left) == 0) {
            return 0;
        }
        let argument_index = node.extra_end;
        let argument_position = 0;
        while (argument_position < node.extra) {
            if (argument_index < 0) {
                checker.error = 4;
                return 0;
            }
            if (argument_index >= expressions.argument_count) {
                checker.error = 4;
                return 0;
            }
            if (semantic_expression(checker, expressions, expressions.arguments[argument_index].expression) == 0) {
                return 0;
            }
            argument_index = expressions.arguments[argument_index].next;
            argument_position = argument_position + 1;
        }
        if (argument_index >= 0) {
            checker.error = 4;
            return 0;
        }
        return 1;
    }
    if (node.kind == 7) {
        if (semantic_expression(checker, expressions, node.left) == 0) {
            return 0;
        }
        return semantic_expression(checker, expressions, node.right);
    }
    if (node.kind == 8) {
        if (semantic_expression(checker, expressions, node.left) == 0) {
            return 0;
        }
        return semantic_expression(checker, expressions, node.right);
    }
    checker.error = 4;
    return 0;
}

int semantic_statement_range(
    struct SemanticChecker* checker,
    struct ExpressionArena* expressions,
    struct StatementArena* statements,
    int start,
    int count
) {
    let index = start;
    let end = start + count;
    while (index < end) {
        if (index >= checker.statement_arena.count) {
            checker.error = 4;
            return 0;
        }
        let statement = &statements.nodes[index];
        if (statement.kind == 1) {
            let value_type = semantic_expression_type(checker, expressions, statement.right);
            if (value_type == 0) {
                return 0;
            }
            if (semantic_declare(checker, statement.left, statement.extra, 1, value_type) == 0) {
                return 0;
            }
        } else {
            if (statement.kind == 2) {
                let actual = semantic_expression_type(checker, expressions, statement.left);
                if (actual == 0) {
                    return 0;
                }
                if (actual != checker.return_type) {
                    checker.error = 8;
                    return 0;
                }
            } else {
                if (statement.kind == 3) {
                    let target = semantic_expression_type(checker, expressions, statement.left);
                    let value = semantic_expression_type(checker, expressions, statement.right);
                    if (target == 0) {
                        return 0;
                    }
                    if (value == 0) {
                        return 0;
                    }
                    if (target != value) {
                        checker.error = 9;
                        return 0;
                    }
                } else {
                    if (statement.kind == 4) {
                        let expression_type = semantic_expression_type(checker, expressions, statement.left);
                        if (expression_type == 0) {
                            return 0;
                        }
                        if (expression_type != 6) {
                            checker.error = 10;
                            return 0;
                        }
                    } else {
                        if (statement.kind == 5) {
                            let expression_type = semantic_expression_type(checker, expressions, statement.left);
                            if (expression_type == 0) {
                                return 0;
                            }
                            if (expression_type != 6) {
                                checker.error = 10;
                                return 0;
                            }
                            if (semantic_statement_range(
                                checker,
                                expressions,
                                statements,
                                statement.body_start,
                                statement.body_count
                            ) == 0) {
                                return 0;
                            }
                            if (statement.else_count > 0) {
                                if (semantic_statement_range(
                                    checker,
                                    expressions,
                                    statements,
                                    statement.else_start,
                                    statement.else_count
                                ) == 0) {
                                    return 0;
                                }
                            }
                        } else {
                            if (statement.kind == 6) {
                            let expression_type = semantic_expression_type(checker, expressions, statement.left);
                            if (expression_type == 0) {
                                return 0;
                            }
                            if (expression_type != 6) {
                                checker.error = 10;
                                return 0;
                            }
                            if (semantic_statement_range(
                                checker,
                                expressions,
                                statements,
                                statement.body_start,
                                statement.body_count
                            ) == 0) {
                                return 0;
                            }
                            if (statement.else_count > 0) {
                                if (semantic_statement_range(
                                    checker,
                                    expressions,
                                    statements,
                                    statement.else_start,
                                    statement.else_count
                                ) == 0) {
                                    return 0;
                                }
                            }
                            } else {
                                if (statement.kind == 7) {
                                    if (semantic_expression_type(checker, expressions, statement.left) == 0) {
                                        return 0;
                                    }
                                    if (semantic_statement_range(
                                        checker,
                                        expressions,
                                        statements,
                                        statement.body_start,
                                        statement.body_count
                                    ) == 0) {
                                        return 0;
                                    }
                                } else {
                                    checker.error = 4;
                                    return 0;
                                }
                            }
                        }
                    }
                }
            }
        }
        index = index + 1;
    }
    return 1;
}

int semantic_check_program(
    struct SemanticChecker* checker,
    struct AstProgram* program,
    struct AstStorage* storage
) {
    let function_index = 0;
    checker.error = 0;
    while (function_index < program.function_count) {
        checker.scope = function_index + 1;
        checker.symbol_count = 0;
        let function = &storage.functions[function_index];
        checker.return_type = function.return_type;
        let parameter_index = 0;
        while (parameter_index < function.parameter_count) {
            let parameter = &storage.parameters[function.parameter_start + parameter_index];
            if (semantic_declare(
                checker,
                parameter.name_start,
                parameter.name_end,
                2,
                parameter.type_kind
            ) == 0) {
                return 0;
            }
            parameter_index = parameter_index + 1;
        }
        if (semantic_statement_range(
            checker,
            checker.expression_arena,
            checker.statement_arena,
            function.body_start,
            function.body_count
        ) == 0) {
            return 0;
        }
        function_index = function_index + 1;
    }
    return 1;
}

int semantic_check_parser(
    struct SemanticChecker* checker,
    struct AstProgram* program,
    struct Parser* parser,
    struct AstStorage* storage
) {
    semantic_bind(checker, &parser.expression_arena, &parser.statement_arena);
    semantic_bind_signatures(
        checker,
        parser.declarations,
        storage.parameters
    );
    if (semantic_collect_signatures(checker, program, storage, parser.declarations) == 0) {
        return 0;
    }
    return semantic_check_program(checker, program, storage);
}

int parse_program(struct Parser* parser) {
    let count = 0;
    let running = 1;
    while (running == 1) {
        if (parser_peek(parser) == 0) {
            running = 0;
        } else {
            if (parser_peek(parser) == 13) {
                if (parse_struct(parser) == 0) {
                    return 0;
                }
            } else {
                if (parse_function(parser) == 0) {
                    return 0;
                }
            }
            count = count + 1;
        }
    }
    return count;
}

int parse_example(struct Parser* parser) {
    return parse_program(parser);
}

struct IrInstruction {
    int kind;
    int operator;
    int type_kind;
    int start;
    int end;
    int left;
    int right;
    int extra;
}

struct IrFunction {
    int start;
    int count;
    int return_type;
    int parameter_start;
    int parameter_count;
    int block_start;
    int block_count;
}

struct IrBlock {
    int start;
    int count;
    int id;
}

struct IrProgram {
    int function_start;
    int function_count;
}

struct IrStorage {
    struct IrInstruction* instructions;
    int instruction_capacity;
    int instruction_count;
    struct IrFunction* functions;
    int function_capacity;
    int function_count;
    struct IrBlock* blocks;
    int block_capacity;
    int block_count;
    int current_block;
    int error;
}

int ir_init(
    struct IrStorage* ir,
    struct IrInstruction* instructions,
    int instruction_capacity,
    struct IrFunction* functions,
    int function_capacity
) {
    ir.instructions = instructions;
    ir.instruction_capacity = instruction_capacity;
    ir.instruction_count = 0;
    ir.functions = functions;
    ir.function_capacity = function_capacity;
    ir.function_count = 0;
    ir.block_capacity = 0;
    ir.block_count = 0;
    ir.current_block = -1;
    ir.error = 0;
    return 1;
}

int ir_block_storage(
    struct IrStorage* ir,
    struct IrBlock* blocks,
    int block_capacity
) {
    ir.blocks = blocks;
    ir.block_capacity = block_capacity;
    ir.block_count = 0;
    ir.current_block = -1;
    return 1;
}

int ir_block_new(struct IrStorage* ir) {
    if (ir.block_count >= ir.block_capacity) {
        ir.error = 7;
        return -1;
    }
    let id = ir.block_count;
    ir.blocks[id].start = -1;
    ir.blocks[id].count = 0;
    ir.blocks[id].id = id;
    ir.block_count = ir.block_count + 1;
    return id;
}

int ir_block_enter(struct IrStorage* ir, int block) {
    if (block < 0) {
        ir.error = 2;
        return 0;
    }
    if (block >= ir.block_count) {
        ir.error = 2;
        return 0;
    }
    ir.current_block = block;
    if (ir.blocks[block].start < 0) {
        ir.blocks[block].start = ir.instruction_count;
    }
    return 1;
}

int ir_block_terminated(struct IrStorage* ir) {
    if (ir.current_block < 0) {
        return 1;
    }
    if (ir.blocks[ir.current_block].count == 0) {
        return 0;
    }
    let last = ir.blocks[ir.current_block].start +
        ir.blocks[ir.current_block].count - 1;
    let kind = ir.instructions[last].kind;
    if (kind == 11) {
        return 1;
    }
    if (kind == 12) {
        return 1;
    }
    if (kind == 13) {
        return 1;
    }
    return 0;
}

int ir_instruction_init(
    struct IrInstruction* instruction,
    int kind,
    int operator,
    int type_kind,
    int left,
    int right
) {
    instruction.kind = kind;
    instruction.operator = operator;
    instruction.type_kind = type_kind;
    instruction.start = 0;
    instruction.end = 0;
    instruction.left = left;
    instruction.right = right;
    instruction.extra = 0;
    return 1;
}

int ir_emit(
    struct IrStorage* ir,
    struct IrInstruction* instruction
) {
    if (ir.instruction_count >= ir.instruction_capacity) {
        ir.error = 1;
        return -1;
    }
    let index = ir.instruction_count;
    ir.instructions[index].kind = instruction.kind;
    ir.instructions[index].operator = instruction.operator;
    ir.instructions[index].type_kind = instruction.type_kind;
    ir.instructions[index].start = instruction.start;
    ir.instructions[index].end = instruction.end;
    ir.instructions[index].left = instruction.left;
    ir.instructions[index].right = instruction.right;
    ir.instructions[index].extra = instruction.extra;
    ir.instruction_count = ir.instruction_count + 1;
    if (ir.current_block >= 0) {
        ir.blocks[ir.current_block].count =
            ir.blocks[ir.current_block].count + 1;
    }
    return index;
}

int ir_emit_node(
    struct IrStorage* ir,
    int kind,
    int operator,
    int type_kind,
    int left,
    int right
) {
    if (ir.instruction_count >= ir.instruction_capacity) {
        ir.error = 1;
        return -1;
    }
    let index = ir.instruction_count;
    ir.instructions[index].kind = kind;
    ir.instructions[index].operator = operator;
    ir.instructions[index].type_kind = type_kind;
    ir.instructions[index].start = 0;
    ir.instructions[index].end = 0;
    ir.instructions[index].left = left;
    ir.instructions[index].right = right;
    ir.instructions[index].extra = 0;
    ir.instruction_count = ir.instruction_count + 1;
    if (ir.current_block >= 0) {
        ir.blocks[ir.current_block].count =
            ir.blocks[ir.current_block].count + 1;
    }
    return index;
}

int ir_emit_call(
    struct IrStorage* ir,
    int type_kind,
    int callee,
    int argument_count
) {
    let index = ir_emit_node(ir, 7, 0, type_kind, callee, 0);
    if (index >= 0) {
        ir.instructions[index].extra = argument_count;
    }
    return index;
}

int ir_emit_jump(struct IrStorage* ir, int target) {
    return ir_emit_node(ir, 13, 0, 0, target, 0);
}

int ir_emit_branch(struct IrStorage* ir, int then_block, int else_block) {
    return ir_emit_node(ir, 12, 0, 6, then_block, else_block);
}

int ir_lower_expression(
    struct IrStorage* ir,
    struct SemanticChecker* checker,
    struct ExpressionArena* expressions,
    int root
) {
    if (root < 0) {
        ir.error = 2;
        return -1;
    }
    if (root >= expressions.count) {
        ir.error = 2;
        return -1;
    }
    let node = &expressions.nodes[root];
    if (node.kind == 1) {
        return ir_emit_node(ir, 3, 0, semantic_expression_type(checker, expressions, root), 0, 0);
    }
    if (node.kind == 2) {
        return ir_emit_node(ir, 1, 0, 1, 0, 0);
    }
    if (node.kind == 3) {
        return ir_emit_node(ir, 2, 0, 6, 0, 0);
    }
    if (node.kind == 4) {
        if (ir_lower_expression(ir, checker, expressions, node.left) < 0) {
            return -1;
        }
        return ir_emit_node(ir, 5, node.operator, semantic_expression_type(checker, expressions, root), node.left, 0);
    }
    if (node.kind == 5) {
        if (ir_lower_expression(ir, checker, expressions, node.left) < 0) {
            return -1;
        }
        if (ir_lower_expression(ir, checker, expressions, node.right) < 0) {
            return -1;
        }
        return ir_emit_node(ir, 6, node.operator, semantic_expression_type(checker, expressions, root), node.left, node.right);
    }
    if (node.kind == 6) {
        if (ir_lower_expression(ir, checker, expressions, node.left) < 0) {
            return -1;
        }
        let argument_index = node.extra_end;
        let argument_position = 0;
        while (argument_position < node.extra) {
            if (argument_index < 0) {
                ir.error = 2;
                return -1;
            }
            if (argument_index >= expressions.argument_count) {
                ir.error = 2;
                return -1;
            }
            if (ir_lower_expression(
                ir,
                checker,
                expressions,
                expressions.arguments[argument_index].expression
            ) < 0) {
                return -1;
            }
            argument_index = expressions.arguments[argument_index].next;
            argument_position = argument_position + 1;
        }
        if (argument_index >= 0) {
            ir.error = 2;
            return -1;
        }
        return ir_emit_call(ir, semantic_expression_type(checker, expressions, root), node.left, node.extra);
    }
    ir.error = 3;
    return -1;
}

int ir_lower_statement_range(
    struct IrStorage* ir,
    struct SemanticChecker* checker,
    struct StatementArena* statements,
    int start,
    int count
) {
    if (start < 0) {
        ir.error = 2;
        return 0;
    }
    let index = start;
    let end = start + count;
    while (index < end) {
        if (index >= statements.count) {
            ir.error = 2;
            return 0;
        }
        let statement = &statements.nodes[index];
        if (statement.kind == 1) {
            if (ir_lower_expression(
                ir,
                checker,
                checker.expression_arena,
                statement.right
            ) < 0) {
                return 0;
            }
            if (ir_emit_node(ir, 8, 0, 0, statement.left, statement.right) < 0) {
                return 0;
            }
        } else {
            if (statement.kind == 2) {
                if (statement.left >= 0) {
                    if (ir_lower_expression(
                        ir,
                        checker,
                        checker.expression_arena,
                        statement.left
                    ) < 0) {
                        return 0;
                    }
                }
                let has_value = 0;
                if (statement.left >= 0) {
                    has_value = 1;
                }
                let result = ir_emit_node(
                    ir,
                    11,
                    0,
                    checker.return_type,
                    statement.left,
                    0
                );
                if (result < 0) {
                    if (has_value == 1) {
                        ir.error = 2;
                    }
                    return 0;
                }
                ir.instructions[result].extra = has_value;
            } else {
                if (statement.kind == 3) {
                    if (ir_lower_expression(
                        ir,
                        checker,
                        checker.expression_arena,
                        statement.right
                    ) < 0) {
                        return 0;
                    }
                    if (ir_emit_node(ir, 9, 0, 0, statement.left, statement.right) < 0) {
                        return 0;
                    }
                } else {
                    if (statement.kind == 4) {
                        if (ir_lower_expression(
                            ir,
                            checker,
                            checker.expression_arena,
                            statement.left
                        ) < 0) {
                            return 0;
                        }
                        if (ir_emit_node(ir, 10, 0, 0, statement.left, 0) < 0) {
                            return 0;
                        }
                    } else {
                        if (statement.kind == 5) {
                            if (ir_lower_statement_range(
                                ir,
                                checker,
                                statements,
                                statement.body_start,
                                statement.body_count
                            ) == 0) {
                                return 0;
                            }
                        } else {
                            if (statement.kind == 6) {
                                let then_block = ir_block_new(ir);
                                let else_block = ir_block_new(ir);
                                let end_block = ir_block_new(ir);
                                if (then_block < 0) {
                                    return 0;
                                }
                                if (else_block < 0) {
                                    return 0;
                                }
                                if (end_block < 0) {
                                    return 0;
                                }
                                if (ir_lower_expression(
                                    ir,
                                    checker,
                                    checker.expression_arena,
                                    statement.left
                                ) < 0) {
                                    return 0;
                                }
                                if (ir_emit_branch(ir, then_block, else_block) < 0) {
                                    return 0;
                                }
                                if (ir_block_enter(ir, then_block) == 0) {
                                    return 0;
                                }
                                if (ir_lower_statement_range(
                                    ir,
                                    checker,
                                    statements,
                                    statement.body_start,
                                    statement.body_count
                                ) == 0) {
                                    return 0;
                                }
                                if (ir_block_terminated(ir) == 0) {
                                    if (ir_emit_jump(ir, end_block) < 0) {
                                        return 0;
                                    }
                                }
                                if (ir_block_enter(ir, else_block) == 0) {
                                    return 0;
                                }
                                if (statement.else_count > 0) {
                                    if (ir_lower_statement_range(
                                        ir,
                                        checker,
                                        statements,
                                        statement.else_start,
                                        statement.else_count
                                    ) == 0) {
                                        return 0;
                                    }
                                }
                                if (ir_block_terminated(ir) == 0) {
                                    if (ir_emit_jump(ir, end_block) < 0) {
                                        return 0;
                                    }
                                }
                                if (ir_block_enter(ir, end_block) == 0) {
                                    return 0;
                                }
                            } else {
                                if (statement.kind == 7) {
                                    let condition_block = ir_block_new(ir);
                                    let body_block = ir_block_new(ir);
                                    let end_block = ir_block_new(ir);
                                    if (condition_block < 0) {
                                        return 0;
                                    }
                                    if (body_block < 0) {
                                        return 0;
                                    }
                                    if (end_block < 0) {
                                        return 0;
                                    }
                                    if (ir_emit_jump(ir, condition_block) < 0) {
                                        return 0;
                                    }
                                    if (ir_block_enter(ir, condition_block) == 0) {
                                        return 0;
                                    }
                                    if (ir_lower_expression(
                                        ir,
                                        checker,
                                        checker.expression_arena,
                                        statement.left
                                    ) < 0) {
                                        return 0;
                                    }
                                    if (ir_emit_branch(ir, body_block, end_block) < 0) {
                                        return 0;
                                    }
                                    if (ir_block_enter(ir, body_block) == 0) {
                                        return 0;
                                    }
                                    if (ir_lower_statement_range(
                                        ir,
                                        checker,
                                        statements,
                                        statement.body_start,
                                        statement.body_count
                                    ) == 0) {
                                        return 0;
                                    }
                                    if (ir_block_terminated(ir) == 0) {
                                        if (ir_emit_jump(ir, condition_block) < 0) {
                                            return 0;
                                        }
                                    }
                                    if (ir_block_enter(ir, end_block) == 0) {
                                        return 0;
                                    }
                                } else {
                                    ir.error = 4;
                                    return 0;
                                }
                            }
                        }
                    }
                }
            }
        }
        index = index + 1;
    }
    return 1;
}

int ir_lower_program(
    struct IrStorage* ir,
    struct SemanticChecker* checker,
    struct AstProgram* program,
    struct AstStorage* storage
) {
    ir.function_count = 0;
    let function_index = 0;
    while (function_index < program.function_count) {
        if (ir.function_count >= ir.function_capacity) {
            ir.error = 5;
            return 0;
        }
        let function = &storage.functions[function_index];
        checker.scope = function_index + 1;
        checker.symbol_count = 0;
        checker.return_type = function.return_type;
        let parameter_index = 0;
        while (parameter_index < function.parameter_count) {
            let parameter = &storage.parameters[function.parameter_start + parameter_index];
            if (semantic_declare(
                checker,
                parameter.name_start,
                parameter.name_end,
                2,
                parameter.type_kind
            ) == 0) {
                ir.error = 6;
                return 0;
            }
            parameter_index = parameter_index + 1;
        }
        let output = &ir.functions[ir.function_count];
        output.start = ir.instruction_count;
        output.count = 0;
        output.return_type = function.return_type;
        output.parameter_start = function.parameter_start;
        output.parameter_count = function.parameter_count;
        ir.block_count = 0;
        ir.current_block = -1;
        let entry_block = ir_block_new(ir);
        if (entry_block < 0) {
            return 0;
        }
        if (ir_block_enter(ir, entry_block) == 0) {
            return 0;
        }
        output.block_start = entry_block;
        if (ir_lower_statement_range(
            ir,
            checker,
            checker.statement_arena,
            function.body_start,
            function.body_count
        ) == 0) {
            return 0;
        }
        output.count = ir.instruction_count - output.start;
        output.block_count = ir.block_count;
        ir.function_count = ir.function_count + 1;
        function_index = function_index + 1;
    }
    return 1;
}
