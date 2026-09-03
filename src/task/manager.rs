use crate::{
    loader::num_apps,
    mm::KERNEL_SPACE,
    sbi::shutdown,
    sync_refcell::SyncRefCell,
    task::{
        TaskContext,
        switch::__switch,
        task::{
            self,
            Status::{Exit, Ready, Running, Suspended},
            TaskControlBlock,
        },
    },
};
use alloc::{sync::Arc, vec::Vec};
use lazy_static::lazy_static;

pub struct TaskManager {
    running_task: SyncRefCell<usize>,
    tasks: SyncRefCell<Vec<Arc<task::TaskControlBlock>>>,
}

pub fn suspend_current() {
    TASK_MANAGER.suspend_current();
}

pub fn exit_current() {
    TASK_MANAGER.exit_current();
}

pub fn run_next_task() {
    TASK_MANAGER.run_next_task()
}

impl TaskManager {
    pub fn current_token(&self) -> usize {
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
    pub(crate) fn run_first_task(&self) -> ! {
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
