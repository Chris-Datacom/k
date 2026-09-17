// Shared lexer conformance fixture.
struct Pair {
    char symbol;
    int value;
}

int main() {
    let message = "K\n";
    let escaped = '\'';
    let value = 0;
    if (value != 0) {
        value = value + 1;
    } else {
        value = value - 1;
    }
    while (value <= 10) {
        value = value * 2;
    }
    if (value >= 1) {
        value = value;
    }
    return value;
}
