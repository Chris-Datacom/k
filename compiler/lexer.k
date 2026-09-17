// Allocation-free source scanner used as the first self-hosting component.

struct Token {
    int kind;
    int start;
    int end;
}

// Token kinds:
// 0 = end of input
// 1 = identifier
// 2 = decimal integer
// 3 = punctuation or operator
// 4 = whitespace
// -1 = invalid input
int lex_one(char* source, int length, int position, struct Token* out) {
    if (position >= length) {
        out.kind = 0;
        out.start = position;
        out.end = position;
        return position;
    }

    let current = source[position];
    out.start = position;
    out.end = position + 1;

    if (current == ' ') {
        out.kind = 4;
        return position + 1;
    }
    if (current == '\n') {
        out.kind = 4;
        return position + 1;
    }
    if (current == '\t') {
        out.kind = 4;
        return position + 1;
    }
    if (current == '\r') {
        out.kind = 4;
        return position + 1;
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

    out.kind = -1;
    return position + 1;
}
