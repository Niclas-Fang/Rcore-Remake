.section .text.trampoline
.globl _trap_entry

    
.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    .align 12
_trap_entry:
    csrrw sp, sscratch, sp

    sd x1, 1*8(sp)
    # skip x2
    sd x3, 3*8(sp)

    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    csrr t0, sstatus
    csrr t1, sepc
    csrr t2, sscratch

    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    sd t2, 2*8(sp)

    ld t0, 34*8(sp)
    ld t1, 37*8(sp)

    mv a0, sp
    ld sp, 36*8(sp)

    csrw satp, t0
    sfence.vma

    la ra, __restore
    jr t1
    
.globl __restore
__restore:
    mv sp, a0
    csrw sscratch, a0

    ld t0,32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    ld t0, 35*8(sp)
    csrw satp, t0
    sfence.vma

    ld sp, 2*8(sp)
    sret
