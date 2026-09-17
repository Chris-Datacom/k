struct Token {
    int kind;
    int start;
    int end;
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

int parse_primary(struct Parser* parser) {
    let next = parser_peek(parser);
    if (next == 1) {
        parser_advance(parser);
        return 1;
    }
    if (next == 2) {
        parser_advance(parser);
        return 1;
    }
    if (next == 3) {
        parser_advance(parser);
        return 1;
    }
    return 0;
}

int parse_expression(struct Parser* parser) {
    return parse_primary(parser);
}

int parse_statement(struct Parser* parser) {
    let consumed = parse_expression(parser);
    if (consumed == 0) {
        return 0;
    }
    if (parser_peek(parser) == 3) {
        parser_advance(parser);
    }
    return 1;
}

int parse_program(struct Parser* parser) {
    let count = 0;
    let loop_running = 1;
    while (loop_running == 1) {
        if (parser_peek(parser) == 0) {
            loop_running = 0;
        } else {
            if (parse_statement(parser) == 0) {
                loop_running = 0;
            } else {
                count = count + 1;
            }
        }
    }
    return count;
}

int parse_example(struct Parser* parser) {
    let count = parse_program(parser);
    if (count > 0) {
        return count;
    }
    return 0;
}
