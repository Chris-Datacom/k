// Self-hosted standalone compiler entry for K.

int main() {
    print(".intel_syntax noprefix\n.text\n");
    print(".globl _start\n_start:\n");
    print("  mov rax, 60\n  mov rdi, 0\n  syscall\n");
    return 0;
}
