use crate::{
    config::{TRAP_CONTEXT, USER_BASE_VA},
    loader::kernel_sp,
    mm::{KERNEL_SPACE, MemorySet, PhyPageNum, VirtAddr, kernel_satp},
    sync_refcell::SyncRefCell,
    task::{
        Status::{Exit, Running},
        switch::__switch,
    },
    trap::{TrapContext, trap_handler},
};
use alloc::vec::Vec;
pub use context::TaskContext;
use lazy_static::lazy_static;

use crate::{
    loader::num_apps,
    sbi::shutdown,
    task::Status::{Ready, Suspended},
};
mod context;
mod switch;
struct Task {
    status: Status,
    context: TaskContext,
    memory_set: MemorySet,
    page_table_token: usize,
    trap_cx_ppn: PhyPageNum,
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
    tasks: SyncRefCell<Vec<Task>>,
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
impl Task {
    pub fn new(app_id: usize) -> Self {
        let memory_set = MemorySet::from_app(app_id);
        let task = Task {
            context: TaskContext::goto_restore(TRAP_CONTEXT),
            status: Status::Ready,
            page_table_token: memory_set.token(),
            trap_cx_ppn: memory_set
                .translate(VirtAddr(TRAP_CONTEXT).floor())
                .unwrap(),
            memory_set: memory_set,
        };
        let cx_ptr = task.trap_cx_ppn.get_bytes_array().as_mut_ptr() as *mut TrapContext;
        unsafe {
            cx_ptr.write(TrapContext::init(
                USER_BASE_VA,
                TRAP_CONTEXT,
                kernel_satp(),
                task.page_table_token,
                kernel_sp(app_id),
                trap_handler as *const () as usize,
            ))
        };
        task
    }
}
impl TaskManager {
    fn current_token(&self) -> usize {
        self.tasks.borrow()[*self.running_task.borrow()].page_table_token
    }
    fn suspend_current(&self) {
        self.tasks.borrow_mut()[*self.running_task.borrow()].status = Suspended;
    }
    fn exit_current(&self) {
        self.tasks.borrow_mut()[*self.running_task.borrow()].status = Exit;
    }
    fn run_next_task(&self) {
        let next_task = self.find_next_task();
        let current_context = {
            let mut tasks = self.tasks.borrow_mut();
            let idx = *self.running_task.borrow();
            &mut tasks[idx].context as *mut TaskContext
        };

        let (next_context, ppn) = {
            let tasks = self.tasks.borrow();
            (
                &tasks[next_task].context as *const TaskContext,
                tasks[next_task].trap_cx_ppn,
            )
        };

        *self.running_task.borrow_mut() = next_task;
        self.tasks.borrow_mut()[next_task].status = Running;
        KERNEL_SPACE.borrow_mut().remap_trap_context(ppn);
        unsafe { __switch(current_context, next_context) };
    }
    fn find_next_task(&self) -> usize {
        let running = *self.running_task.borrow();
        let tasks = self.tasks.borrow();
        let app_num = self.tasks.borrow().len();
        for i in 1..app_num + 1 {
            let idx = (running + i) % app_num;
            let status = tasks[idx].status;
            if status == Ready || status == Suspended {
                return idx;
            }
        }
        shutdown();
    }
    fn run_first_task(&self) -> ! {
        let first_cx_ptr: *const TaskContext;
        {
            let mut tasks = self.tasks.borrow_mut();
            tasks[0].status = Running;
            first_cx_ptr = &tasks[0].context as *const TaskContext;
            KERNEL_SPACE
                .borrow_mut()
                .remap_trap_context(tasks[0].trap_cx_ppn);
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
        let tasks: Vec<_> = (0..app_num).map(Task::new).collect();
        unsafe {
            TaskManager {
                tasks: SyncRefCell::new(tasks),
                running_task: SyncRefCell::new(0),
            }
        }
    };
}
