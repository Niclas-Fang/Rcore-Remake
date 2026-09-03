#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(static_mut_refs)]

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

global_asm!(include_str!("entry.S"));

unsafe extern "C" {
    unsafe static sbss: u8;
    unsafe static ebss: u8;
}

#[unsafe(no_mangle)]
unsafe extern "C" fn kernel_main() -> ! {
    logging::init();
    println!("Hello world from main!");
    loader::list_apps();
    mm::init_heap();
    mm::init_frame_allocator();
    mm::activate();
    trap::init();
    timer::init_timer();
    task::run_tasks()
}
