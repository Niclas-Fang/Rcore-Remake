#![no_std]
#![no_main]
#![allow(dead_code)]

mod sbi;
use core::arch::global_asm;
use core::panic::PanicInfo;

global_asm!(include_str!("entry.S"));


unsafe extern "C" {
    unsafe static sbss: u8;
    unsafe static ebss: u8;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_main() -> ! {
    sbi::puts("Hello, world!\n");
    sbi::shutdown();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    sbi::puts("Kernel panic!\n");
    sbi::shutdown();
}