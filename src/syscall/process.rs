use log::trace;
use riscv::register::time;

use crate::task::{
    add_task, current_task, exit_current_and_run_next, suspend_current_and_run_next,
};
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] sys_exit with code {exit_code}");
    exit_current_and_run_next(exit_code)
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

pub fn sys_fork() -> usize {
    let parent = current_task().expect("There is no task task running!");
    let child = parent.fork();
    let pid = child.pid.0;
    add_task(child);
    pid
}
