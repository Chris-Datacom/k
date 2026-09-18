// Allocation-free source scanner used as the first self-hosting component.

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

// Token kinds:
// 0 = end of input
// 1 = identifier
// 2 = decimal integer
// 3 = punctuation or operator
// 4..13 = keywords
// -1 = invalid input
int lex_one(char* source, int length, int position, struct Token* out) {
    let scanning = true;
    while (scanning) {
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
                                        let comment_scanning = true;
                                        while (comment_scanning) {
                                            if (position < length) {
                                            if (source[position] != '\n') {
                                                position = position + 1;
                                            } else {
                                                comment_scanning = false;
                                            }
                                            } else {
                                                comment_scanning = false;
                                            }
                                        }
                                    } else {
                                        scanning = false;
                                    }
                                } else {
                                    scanning = false;
                                }
                            } else {
                                scanning = false;
                            }
                        }
                    }
                }
            }
        } else {
            scanning = false;
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
        let scanning = 1;
        while (scanning == 1) {
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
                        scanning = 0;
                    } else {
                        next = next + 1;
                    }
                }
            } else {
                scanning = 0;
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
            let scanning = true;
            while (scanning) {
                if (next < length) {
                    let digit = source[next];
                    if (digit >= '0') {
                        if (digit <= '9') {
                            next = next + 1;
                        } else {
                            scanning = false;
                        }
                    } else {
                        scanning = false;
                    }
                } else {
                    scanning = false;
                }
            }
            out.kind = 2;
            out.end = next;
            return next;
        }
    }

    let identifier_start = false;
    if (current == '_') {
        identifier_start = true;
    }
    if (current >= 'a') {
        if (current <= 'z') {
            identifier_start = true;
        }
    }
    if (current >= 'A') {
        if (current <= 'Z') {
            identifier_start = true;
        }
    }
    if (identifier_start) {
        let next = position + 1;
        let scanning = true;
        while (scanning) {
            if (next < length) {
                let character = source[next];
                let identifier_part = false;
                if (character == '_') {
                    identifier_part = true;
                }
                if (character >= 'a') {
                    if (character <= 'z') {
                        identifier_part = true;
                    }
                }
                if (character >= 'A') {
                    if (character <= 'Z') {
                        identifier_part = true;
                    }
                }
                if (character >= '0') {
                    if (character <= '9') {
                        identifier_part = true;
                    }
                }
                if (identifier_part) {
                    next = next + 1;
                } else {
                    scanning = false;
                }
            } else {
                scanning = false;
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
        }
        if (next - position == 6) {
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
            if (source[position + 1] == '<') {
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
            if (source[position + 1] == '>') {
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
    if (current == '|') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '^') {
        out.kind = 3;
        return position + 1;
    }
    if (current == '~') {
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
