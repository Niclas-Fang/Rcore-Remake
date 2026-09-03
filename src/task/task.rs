use crate::{task::vec, trap::TrapContext};
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

use crate::{
    config::{TRAP_CONTEXT, USER_BASE_VA},
    mm::{MemorySet, PhyPageNum, VirtAddr, kernel_satp},
    sync_refcell::SyncRefCell,
    task::{
        TaskContext,
        pid::{self, KernelStack, PidHandle},
    },
    trap::trap_handler,
};

pub struct TaskControlBlockInner {
    pub status: Status,
    pub context: TaskContext,
    memory_set: MemorySet,
    pub page_table_token: usize,
    pub trap_cx_ppn: PhyPageNum,
    parent: Option<Weak<TaskControlBlock>>,
    children: Vec<Arc<TaskControlBlock>>,
    exit_code: i32,
    base_size: usize,
}
pub struct TaskControlBlock {
    pub pid: PidHandle,
    pub kernel_stack: KernelStack,
    pub inner: SyncRefCell<TaskControlBlockInner>,
}
#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Ready,
    Running,
    Suspended,
    Exit,
}

impl TaskControlBlock {
    pub fn new(app_id: usize) -> Self {
        let pid = pid::pid_alloc();
        let kernel_stack = KernelStack::new(pid.0);

        let memory_set = MemorySet::from_app(app_id);
        let task_inner = TaskControlBlockInner {
            context: TaskContext::goto_restore(TRAP_CONTEXT),
            status: Status::Ready,
            page_table_token: memory_set.token(),
            trap_cx_ppn: memory_set
                .translate(VirtAddr(TRAP_CONTEXT).floor())
                .unwrap(),
            memory_set: memory_set,
            parent: None,
            children: vec![],
            base_size: USER_BASE_VA,
            exit_code: 0,
        };
        let cx_ptr = task_inner.trap_cx_ppn.get_bytes_array().as_mut_ptr() as *mut TrapContext;
        unsafe {
            cx_ptr.write(TrapContext::init(
                USER_BASE_VA,
                TRAP_CONTEXT,
                kernel_satp(),
                task_inner.page_table_token,
                kernel_stack.sp(),
                trap_handler as *const () as usize,
            ));
        };
        TaskControlBlock {
            pid,
            kernel_stack,
            inner: unsafe { SyncRefCell::new(task_inner) },
        }
    }
}
