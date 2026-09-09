use alloc::sync::Arc;
use lazy_static::lazy_static;

use crate::{
    mm::remap_trap_context,
    sbi::shutdown,
    sync_refcell::SyncRefCell,
    task::{
        TaskContext,
        control_block::{Status::Running, TaskControlBlock},
        manager::fetch_task,
        switch::__switch,
    },
};

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(Arc::clone)
    }
    pub fn idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut TaskContext
    }
    pub fn token(&self) -> Option<usize> {
        self.current
            .as_ref()
            .map(|x| x.inner.borrow().page_table_token)
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.borrow_mut().take_current()
}
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    PROCESSOR.borrow().current()
}
pub fn idle_task_cx_ptr() -> *mut TaskContext {
    PROCESSOR.borrow_mut().idle_task_cx_ptr()
}
pub fn token() -> Option<usize> {
    PROCESSOR.borrow().token()
}
pub fn schedule(cx: *mut TaskContext) {
    let mut processor = PROCESSOR.borrow_mut();
    let idle_cx = processor.idle_task_cx_ptr();
    drop(processor);
    unsafe { __switch(cx, idle_cx) };
}
pub fn run_tasks() -> ! {
    loop {
        let Some(task) = fetch_task() else { shutdown() };
        let mut processor = PROCESSOR.borrow_mut();
        let mut task_inner = task.inner.borrow_mut();
        let next_task_cx_ptr = &task_inner.context as *const _;
        let idle_task_cx_ptr = processor.idle_task_cx_ptr();
        let trap_cx_ppn = task_inner.trap_cx_ppn;
        task_inner.status = Running;
        drop(task_inner);
        processor.current = Some(task);
        drop(processor);
        remap_trap_context(trap_cx_ppn);
        unsafe {
            __switch(idle_task_cx_ptr, next_task_cx_ptr);
        }
    }
}

lazy_static! {
    pub static ref PROCESSOR: SyncRefCell<Processor> = unsafe {
        SyncRefCell::new(Processor {
            current: None,
            idle_task_cx: TaskContext::init(),
        })
    };
}
