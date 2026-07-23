pub use context::TaskContext;
use lazy_static::lazy_static;

use crate::loader::{MAX_NUM_APP, init_app_cx, num_apps};
mod context;
mod switch;
#[derive(Clone, Copy)]
struct Task {
    status: Status,
    context: TaskContext,
}
#[derive(Clone, Copy)]
enum Status {
    Ready,
    Running,
    Suspended,
    Exit,
}

pub struct TaskManager {
    running_task: usize,
    tasks: [Task; 16],
    app_num: usize,
}

fn suspend_current() {
    TASK_MANAGER.suspend_current();
}

fn exit_current() {
    TASK_MANAGER.exit_current();
}

fn run_next_task() -> ! {
    TASK_MANAGER.run_next_task()
}

pub fn exit_current_and_run_next() -> ! {
    exit_current();
    run_next_task()
}

pub fn suspend_current_and_run_next() -> ! {
    suspend_current();
    run_next_task()
}

impl TaskManager {
    fn suspend_current(&self) {}
    fn exit_current(&self) {}
    fn run_next_task(&self) -> ! {
        unimplemented!()
    }
    fn find_next_task(&self) -> usize {
        unimplemented!()
    }
}
lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let app_num = num_apps();
        let mut tasks = [Task {
            context: TaskContext::init(),
            status: Status::Ready,
        }; MAX_NUM_APP];
        for (i, task) in tasks.iter_mut().enumerate() {
            task.context = TaskContext::goto_restore(init_app_cx(i));
            task.status = Status::Ready;
        }
        TaskManager {
            app_num,
            tasks,
            running_task: 0,
        }
    };
}
