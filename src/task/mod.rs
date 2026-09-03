pub use context::TaskContext;
use manager::{TASK_MANAGER, exit_current, run_next_task, suspend_current};

mod context;
mod manager;
mod pid;
mod switch;
mod task;
pub fn exit_current_and_run_next() -> ! {
    exit_current();
    run_next_task();
    unreachable!();
}

pub fn suspend_current_and_run_next() {
    suspend_current();
    run_next_task();
}

pub fn run_first_task() -> ! {
    TASK_MANAGER.run_first_task()
}

/// 当前运行任务的用户页表 token（供 syscall 翻译用户地址用）
pub fn current_user_token() -> usize {
    TASK_MANAGER.current_token()
}
