#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(static_mut_refs)]

//mod batch;
mod config;
mod console;
mod lang_items;
mod link_app;
mod loader;
mod logging;
mod mm;
mod sbi;
mod sync_refcell;
mod syscall;
mod task;
mod timer;
mod trap;
use core::arch::global_asm;
extern crate alloc;
use crate::{loader::load_apps, mm::kernel_satp, task::run_first_task};

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

    mm::init_heap();
    mm::init_frame_allocator();
    println!("{}", kernel_satp());
    load_apps();
    timer::init_timer();
    run_first_task()
}
