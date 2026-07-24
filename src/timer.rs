use crate::sbi::set_timer;
use riscv::register::{sie::set_stimer, time};

const CLOCK_FREQ: usize = 10_000_000;
const TISK_PER_SEC: usize = 100;

pub fn set_trigger() {
    set_timer(time::read() + CLOCK_FREQ / TISK_PER_SEC);
}

pub fn init_timer() {
    unsafe { set_stimer() };
    set_trigger();
}
