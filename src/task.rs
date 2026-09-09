pub use context::TaskContext;
pub use init::add_initproc;
pub use manager::add_task;
pub use processor::current_task;
pub use processor::run_tasks;
use {
    control_block::Status::{Ready, Zombie},
    processor::{schedule, take_current_task, token},
};

mod context;
mod control_block;
mod init;
mod manager;
mod pid;
mod processor;
mod switch;

pub fn exit_current_and_run_next(code: i32) -> ! {
    let task = take_current_task().unwrap();
    // task未drop, 稍后防泄漏
    let mut lock = task.inner.borrow_mut();
    lock.status = Zombie;
    lock.exit_code = code;
    drop(lock);
    let mut cx = TaskContext::init();
    schedule(&mut cx as *mut _);
    unreachable!();
}

pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut lock = task.inner.borrow_mut();
    lock.status = Ready;
    let cx = &mut lock.context as *mut _;
    drop(lock);
    add_task(task);
    schedule(cx);
}

pub fn current_user_token() -> usize {
    token().expect("There is no task running!")
}
