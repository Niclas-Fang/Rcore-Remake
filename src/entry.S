.section .bss.stack
.globl sstack
sstack:
    .space 4096 * 16
.globl estack
estack:

.section .text.entry
.globl _start
_start:
    la sp, estack
    la t0, sbss
    la t1, ebss

1:
    bgeu t0, t1, 2f       
    sd   zero, 0(t0)      
    addi t0, t0, 8
    j    1b

2:
    call kernel_main
    j    .