OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY {
    RAM (rwx) : ORIGIN = 0x80200000, LENGTH = 128M
}

SECTIONS {
    . = ORIGIN(RAM);
    skernel = .;
    . = ALIGN(4K);

    stext = .;
    .text : {
        KEEP(*(.text.entry))
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline)
        . = ALIGN(4K);
        etrampoline = .;
        *(.text .text.*)
    } > RAM
    . = ALIGN(4K);
    etext = .;

    srodata = .; 
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    } > RAM
    . = ALIGN(4K);
    erodata = .;

    sdata = .; 
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    } > RAM
    . = ALIGN(4K);
    edata = .; 

    
    sbss = .; 
    .bss : {
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
