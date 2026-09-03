use crate::task::{
    manager::add_task,
    processor::{schedule, take_current_task, token},
    task::Status::{Ready, Zombie},
};
pub use context::TaskContext;
pub use processor::run_tasks;

mod context;
mod manager;
mod pid;
mod processor;
mod switch;
mod task;

pub fn exit_current_and_run_next(code: i32) -> ! {
    let task = take_current_task().unwrap();
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
