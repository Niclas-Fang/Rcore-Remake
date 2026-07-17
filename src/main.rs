#![no_std]
#![no_main]

mod sbi;
use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("entry.S"));


unsafe extern "C" {
    unsafe static _stack_start: u8;
    unsafe static _bss_start: u8;
    unsafe static _bss_end: u8;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_main() -> ! {
    sbi::puts("Hello, world!\n");
    sbi::shutdown();
    //loop{}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sbi::puts("Kernel panic!\n");
    sbi::shutdown();
}