/* Kernel image layout for the x86_64-krumpyos target.
 *
 * The boot stub must provide the _start symbol.  It is deliberately not
 * supplied by the compiler backend: the stub owns CPU setup and the initial
 * stack before it transfers control to K code.
 */
OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5);   /* readable, executable */
    rodata PT_LOAD FLAGS(4); /* readable */
    data PT_LOAD FLAGS(6);   /* readable, writable */
}

SECTIONS
{
    /* The BIOS stub loads the kernel payload at physical address 0x10000. */
    . = 0x00010000;
    __kernel_start = .;

    .text ALIGN(16) : 
    {
        __text_start = .;
        KEEP(*(.text.boot))
        *(.text .text.*)
        __text_end = .;
    } :text

    .rodata ALIGN(16) :
    {
        __rodata_start = .;
        *(.rodata .rodata.*)
        __rodata_end = .;
    } :rodata

    .data ALIGN(16) :
    {
        __data_start = .;
        *(.data .data.*)
        __data_end = .;
    } :data

    .bss ALIGN(16) (NOLOAD) :
    {
        __bss_start = .;
        *(COMMON)
        *(.bss .bss.*)
        . = ALIGN(16);
        __bss_end = .;
    } :data

    __kernel_end = .;

    /DISCARD/ :
    {
        *(.eh_frame)
        *(.note.GNU-stack)
        *(.comment)
    }
}
