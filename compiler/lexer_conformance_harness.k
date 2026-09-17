// Runtime check for the allocation-free K lexer contract.
int lexer_conformance(char* source, struct Token* token) {
    let position = 0;

    position = lex_one(source, 11, position, token);
    if (token.kind != 6) {
        return 1;
    }
    if (token.start != 0) {
        return 1;
    }
    if (token.end != 3) {
        return 1;
    }

    position = lex_one(source, 11, position, token);
    if (token.kind != 1) {
        return 1;
    }
    if (token.start != 4) {
        return 1;
    }
    if (token.end != 5) {
        return 1;
    }

    position = lex_one(source, 11, position, token);
    if (token.kind != 3) {
        return 1;
    }
    if (token.start != 6) {
        return 1;
    }
    if (token.end != 7) {
        return 1;
    }

    position = lex_one(source, 11, position, token);
    if (token.kind != 2) {
        return 1;
    }
    if (token.start != 8) {
        return 1;
    }
    if (token.end != 10) {
        return 1;
    }

    position = lex_one(source, 11, position, token);
    if (token.kind != 3) {
        return 1;
    }
    if (token.start != 10) {
        return 1;
    }
    if (token.end != 11) {
        return 1;
    }

    position = lex_one(source, 11, position, token);
    if (token.kind != 0) {
        return 1;
    }
    if (token.start != 11) {
        return 1;
    }
    if (token.end != 11) {
        return 1;
    }
    return 0;
}
