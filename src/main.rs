#![no_std]
#![no_main]
#![allow(dead_code)]

mod batch;
mod console;
mod lang_items;
mod link_app;
mod loader;
mod logging;
mod sbi;
mod sync_refcell;
mod syscall;
mod task;
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
    trap::init();
    println!("Hello world from main!");

    log::error!("This is an error");
    log::warn!("This is a warning");
    log::info!("This is info");
    log::debug!("This is debug info");
    log::trace!("This is a trace");

    batch::run_next_app();
}
