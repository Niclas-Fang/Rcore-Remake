use crate::{
    mm::KERNEL_SPACE,
    sync_refcell::SyncRefCell,
    task::{
        switch::__switch,
        task::{
            Status::{Exit, Running},
            TaskControlBlock,
        },
    },
};
use alloc::vec;
use alloc::{sync::Arc, vec::Vec};
pub use context::TaskContext;
use lazy_static::lazy_static;

use crate::{
    loader::num_apps,
    sbi::shutdown,
    task::task::Status::{Ready, Suspended},
};
mod context;
mod pid;
mod switch;
mod task;
pub struct TaskManager {
    running_task: SyncRefCell<usize>,
    tasks: SyncRefCell<Vec<Arc<task::TaskControlBlock>>>,
}

fn suspend_current() {
    TASK_MANAGER.suspend_current();
}

fn exit_current() {
    TASK_MANAGER.exit_current();
}

pub fn run_next_task() {
    TASK_MANAGER.run_next_task()
}

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
impl TaskManager {
    fn current_token(&self) -> usize {
        self.tasks.borrow()[*self.running_task.borrow()]
            .inner
            .borrow()
            .page_table_token
    }
    fn suspend_current(&self) {
        self.tasks.borrow_mut()[*self.running_task.borrow()]
            .inner
            .borrow_mut()
            .status = Suspended;
    }
    fn exit_current(&self) {
        self.tasks.borrow_mut()[*self.running_task.borrow()]
            .inner
            .borrow_mut()
            .status = Exit;
    }
    fn run_next_task(&self) {
        let next_task = self.find_next_task();
        let current_context = {
            let tasks = self.tasks.borrow_mut();
            let idx = *self.running_task.borrow();
            let current_context = &mut tasks[idx].inner.borrow_mut().context as *mut TaskContext;
            current_context
        };

        let (next_context, ppn) = {
            let tasks = self.tasks.borrow();
            let next_context = &tasks[next_task].inner.borrow().context as *const TaskContext;
            (next_context, tasks[next_task].inner.borrow().trap_cx_ppn)
        };

        *self.running_task.borrow_mut() = next_task;
        self.tasks.borrow_mut()[next_task].inner.borrow_mut().status = Running;
        KERNEL_SPACE.borrow_mut().remap_trap_context(ppn);
        unsafe { __switch(current_context, next_context) };
    }
    fn find_next_task(&self) -> usize {
        let running = *self.running_task.borrow();
        let tasks = self.tasks.borrow();
        let app_num = self.tasks.borrow().len();
        for i in 1..app_num + 1 {
            let idx = (running + i) % app_num;
            let status = tasks[idx].inner.borrow().status;
            if status == Ready || status == Suspended {
                return idx;
            }
        }
        shutdown();
    }
    fn run_first_task(&self) -> ! {
        let first_cx_ptr: *const TaskContext;
        {
            let tasks = self.tasks.borrow();
            tasks[0].inner.borrow_mut().status = Running;
            first_cx_ptr = &tasks[0].inner.borrow_mut().context as *const TaskContext;
            KERNEL_SPACE
                .borrow_mut()
                .remap_trap_context(tasks[0].inner.borrow().trap_cx_ppn);
        }
        let mut place_holder = TaskContext::init();
        unsafe {
            __switch(&mut place_holder as *mut TaskContext, first_cx_ptr);
        }
        unreachable!()
    }
}
lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let app_num = num_apps();
        let tasks: Vec<_> = (0..app_num)
            .map(TaskControlBlock::new)
            .map(Arc::new)
            .collect();
        unsafe {
            TaskManager {
                tasks: SyncRefCell::new(tasks),
                running_task: SyncRefCell::new(0),
            }
        }
    };
}
