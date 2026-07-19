#![no_std]
#![no_main]
#![allow(dead_code)]

mod console;
mod lang_items;
mod logging;
mod sbi;
mod trap;
use core::arch::global_asm;

global_asm!(include_str!("entry.S"));

unsafe extern "C" {
    unsafe static sbss: u8;
    unsafe static ebss: u8;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_main() -> ! {
    logging::init();
    println!("Helloworld from main!");

    log::error!("This is an error");
    log::warn!("This is a warning");
    log::info!("This is info");
    log::debug!("This is debug info");
    log::trace!("This is a trace");

    sbi::shutdown();
}
