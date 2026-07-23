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

pub fn suspend_current() {}

pub fn exit_current() {}

pub fn run_next_task() -> ! {
    unimplemented!()
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
