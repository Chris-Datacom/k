// A minimal freestanding I/O-port smoke test for the x86_64-krumpyos target.
// It writes a byte to a hardware port and reads one back, without any
// operating system, libc, or runtime support.
void write_serial(u16 port, u8 value) {
    outb(port, value);
}

u8 read_serial(u16 port) {
    return inb(port);
}

// Reinterpreting a fixed address as a pointer is how freestanding code
// reaches memory-mapped hardware, such as the VGA text-mode buffer at
// physical address 753664 (0xb8000). Integer<->pointer casts make that
// address usable without inventing a special "raw pointer literal" syntax.
// (K does not yet have hexadecimal integer literals or bitwise operators,
// so the address and the character/attribute packing below use decimal
// arithmetic instead.)
//
// The buffer is `volatile` because the CPU treats it as ordinary memory but
// the video hardware observes every write: the compiler must never merge,
// reorder, or elide these accesses the way it safely could for a plain
// stack slot.
volatile u16* vga_text_buffer() {
    return (volatile u16*)753664;
}

void vga_put_char(int column, int row, u8 character, u8 attribute) {
    let buffer = (volatile u16*)753664;
    let cell_index = row * 80 + column;
    let packed = (int)character + (int)attribute * 256;
    buffer[cell_index] = (u16)packed;
}
