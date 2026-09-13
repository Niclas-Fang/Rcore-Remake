use log::trace;
use riscv::register::time;

use crate::{
    loader::get_app_data_by_name,
    mm::str_from_path,
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
    },
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

pub fn sys_exec(path: usize) -> isize {
    let token = current_user_token();
    let name = str_from_path(token, path);
    let Some(bin) = get_app_data_by_name(&name) else {
        return -1;
    };
    let task = current_task().expect("There is no task task running!");
    task.exec(bin);
    0
}
