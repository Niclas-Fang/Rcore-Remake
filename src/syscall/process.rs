use crate::task::{exit_current, run_next_task, suspend_current};
pub fn sys_exit(exit_code: i32) -> ! {
    crate::println!("[kernel] sys_exit with code {exit_code}");
    exit_current();
    run_next_task()
}

pub fn sys_yield() -> ! {
    suspend_current();
    run_next_task()
}
