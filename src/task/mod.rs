use crate::{
    sync_refcell::SyncRefCell,
    task::{Status::Exit, switch::__switch},
};
pub use context::TaskContext;
use lazy_static::lazy_static;

use crate::{
    loader::{MAX_NUM_APP, init_app_cx, num_apps},
    sbi::shutdown,
    task::Status::{Ready, Suspended},
};
mod context;
mod switch;
#[derive(Clone, Copy)]
struct Task {
    status: Status,
    context: TaskContext,
}
#[derive(Clone, Copy, PartialEq)]
enum Status {
    Ready,
    Running,
    Suspended,
    Exit,
}

pub struct TaskManager {
    running_task: SyncRefCell<usize>,
    tasks: SyncRefCell<[Task; MAX_NUM_APP]>,
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
    fn suspend_current(&self) {
        self.tasks.borrow_mut()[*self.running_task.borrow()].status = Suspended;
    }
    fn exit_current(&self) {
        self.tasks.borrow_mut()[*self.running_task.borrow()].status = Exit;
    }
    fn run_next_task(&self) -> ! {
        let next_task = self.find_next_task();
        let current_context =
            &mut self.tasks.borrow_mut()[*self.running_task.borrow()].context as *mut TaskContext;
        let next_context = &self.tasks.borrow()[next_task].context as *const TaskContext;
        *self.running_task.borrow_mut() = next_task;
        unsafe { __switch(current_context, next_context) };
    }
    fn find_next_task(&self) -> usize {
        let running = *self.running_task.borrow();
        let tasks = self.tasks.borrow();
        for i in 0..self.app_num {
            let idx = (running + i) % self.app_num;
            let status = tasks[idx].status;
            if status == Ready || status == Suspended {
                return idx;
            }
        }
        shutdown();
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
        unsafe {
            TaskManager {
                app_num,
                tasks: SyncRefCell::new(tasks),
                running_task: SyncRefCell::new(0),
            }
        }
    };
}
