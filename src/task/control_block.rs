use super::{
    TaskContext,
    pid::{self, KernelStack, PidHandle},
};
use crate::{
    config::{TRAP_CONTEXT, USER_BASE_VA},
    mm::{MemorySet, PhyPageNum, VirtAddr, kernel_satp},
    sync_refcell::SyncRefCell,
    task::pid::pid_alloc,
    trap::{TrapContext, trap_handler},
};
use alloc::vec;
use alloc::{
    sync::{Arc, Weak},
    vec::Vec,
};

pub struct TaskControlBlockInner {
    pub status: Status,
    pub context: TaskContext,
    memory_set: MemorySet,
    pub page_table_token: usize,
    pub trap_cx_ppn: PhyPageNum,
    parent: Option<Weak<TaskControlBlock>>,
    children: Vec<Arc<TaskControlBlock>>,
    pub exit_code: i32,
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
    Zombie,
}

impl TaskControlBlock {
    pub fn new(bin: &[u8]) -> Self {
        let pid = pid::pid_alloc();
        let kernel_stack = KernelStack::new(pid.0);

        let memory_set = MemorySet::from_bin(bin);
        let task_inner = TaskControlBlockInner {
            context: TaskContext::goto_restore(TRAP_CONTEXT),
            status: Status::Ready,
            page_table_token: memory_set.token(),
            trap_cx_ppn: memory_set
                .translate(VirtAddr(TRAP_CONTEXT).floor())
                .unwrap(),
            memory_set,
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
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let memory_set = MemorySet::from_parent(&self.inner.borrow_mut().memory_set);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr(TRAP_CONTEXT).floor())
            .unwrap();
        let pid = pid_alloc();
        let kernel_stack = KernelStack::new(pid.0);
        let page_table_token = memory_set.token();
        let trap_cx = unsafe { ((trap_cx_ppn.0 << 12) as *mut TrapContext).as_mut_unchecked() };
        trap_cx.kernel_sp = kernel_stack.sp();
        trap_cx.x[10] = 0;
        trap_cx.user_satp = page_table_token;

        let inner = TaskControlBlockInner {
            context: TaskContext::goto_restore(TRAP_CONTEXT),
            status: Status::Ready,
            parent: Some(Arc::downgrade(self)),
            memory_set,
            page_table_token,
            trap_cx_ppn,
            children: vec![],
            exit_code: 0,
            base_size: USER_BASE_VA,
        };
        let child = unsafe {
            Arc::new(TaskControlBlock {
                pid,
                kernel_stack,
                inner: SyncRefCell::new(inner),
            })
        };
        self.inner.borrow_mut().children.push(child.clone());
        child
    }
}
