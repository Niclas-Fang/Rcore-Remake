use core::arch::asm;

pub fn putchar(c: u8) {
    unsafe {
        asm!(
            "ecall",
            in("a7") 0x4442434Eu64, // syscall number for putchar
            in("a6") 2u64
            in("a0") c as u64, // character to print
        )
    }
}

pub fn puts(s: &str) {
    for &byte in s.bytes() {
        putchar(byte);
    }
}

pub fn shutdown() -> ! {
    unsafe {
        asm!(
            "ecall",
            in("a7") 0x53525354u64, // syscall number for shutdown
            in("a6") 0u64,
            in("a0") 0u64, // exit code
        )
    }
    loop{}
}