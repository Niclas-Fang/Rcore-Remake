OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY {
    /* KERNEL_BASE=0x80200000, KERNEL_MEMORY_SIZE=128M — 见 src/config.rs */
    RAM (rwx) : ORIGIN = 0x80200000, LENGTH = 128M
}

SECTIONS {
    . = ORIGIN(RAM);
    skernel = .;

    stext = .;
    .text : {
        KEEP(*(.text.entry))
        *(.text .text.*)
    } > RAM
    . = ALIGN(4K);
    etext = .;

    srodata = .; 
    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > RAM
    . = ALIGN(4K);
    erodata = .;

    sdata = .; 
    .data : ALIGN(4K) {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } > RAM
    . = ALIGN(4K);
    edata = .; 

    
    sbss = .; 
    .bss : ALIGN(4K) {
        *(.bss.stack)
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    } > RAM
    . = ALIGN(4K);
    ebss = .;

    ekernel = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.comment*)
    }
}
