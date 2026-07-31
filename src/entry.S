# 内核启动栈: 64KiB (见 src/config.rs)
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
    csrw sscratch,sp
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