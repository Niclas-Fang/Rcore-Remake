use crate::{
    config::{CLOCK_FREQ, TICK_PER_SEC},
    sbi::set_timer,
};
use riscv::register::{sie::set_stimer, time};

pub fn set_trigger() {
    set_timer(time::read() + CLOCK_FREQ / TICK_PER_SEC);
}

pub fn init_timer() {
    unsafe { set_stimer() };
    set_trigger();
}
