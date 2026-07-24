use log::trace;
use riscv::register::time;

use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] sys_exit with code {exit_code}");
    exit_current_and_run_next()
}

pub fn sys_yield() -> usize {
    trace!("[kernel] sys_yield");
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> usize {
    trace!("[kernel] sys_get_time");
    time::read()
}
