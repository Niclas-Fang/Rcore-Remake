use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
pub fn sys_exit(exit_code: i32) -> ! {
    crate::println!("[kernel] sys_exit with code {exit_code}");
    exit_current_and_run_next()
}

pub fn sys_yield() -> ! {
    suspend_current_and_run_next()
}
