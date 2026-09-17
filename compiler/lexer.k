// First self-hosting lexer milestone.
//
// The caller owns the source buffer and output token. No allocation or
// runtime support is required. This scanner emits one source unit at a time;
// the next milestone will add token-span loops and keyword recognition.

struct Token {
    int kind;
    int start;
    int end;
}

// Token kinds:
// 0 = end of input
// 1 = identifier character
// 2 = decimal digit
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
            out.kind = 2;
            return position + 1;
        }
    }

    if (current == '_') {
        out.kind = 1;
        return position + 1;
    }
    if (current >= 'a') {
        if (current <= 'z') {
            out.kind = 1;
            return position + 1;
        }
    }
    if (current >= 'A') {
        if (current <= 'Z') {
            out.kind = 1;
            return position + 1;
        }
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
