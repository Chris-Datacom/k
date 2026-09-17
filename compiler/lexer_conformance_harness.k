// Runtime check for the allocation-free K lexer contract.
int expect_token(
    char* source,
    int position,
    struct Token* token,
    int kind,
    int start,
    int end
) {
    let next = lex_one(source, 45, position, token);
    if (token.kind != kind) {
        return -1;
    }
    if (token.start != start) {
        return -1;
    }
    if (token.end != end) {
        return -1;
    }
    return next;
}

int lexer_conformance(char* source, struct Token* token) {
    let position = 0;
    position = expect_token(source, position, token, 6, 0, 3);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 1, 4, 5);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 6, 7);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 2, 8, 10);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 10, 11);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 4, 17, 19);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 20, 21);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 1, 21, 22);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 23, 25);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 2, 26, 27);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 27, 28);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 29, 30);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 1, 31, 32);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 33, 34);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 1, 35, 36);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 37, 39);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 2, 40, 42);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 42, 43);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 3, 44, 45);
    if (position < 0) { return 1; }
    position = expect_token(source, position, token, 0, 45, 45);
    if (position < 0) { return 1; }
    return 0;
}
