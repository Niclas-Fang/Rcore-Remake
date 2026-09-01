use crate::{
    config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE},
    mm::{KERNEL_SPACE, MapPermission, MapType, VirtAddr},
    sync_refcell::SyncRefCell,
};
use alloc::vec::Vec;
use lazy_static::lazy_static;
pub struct PidHandle(pub usize);
pub struct PidAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl PidAllocator {
    fn alloc(&mut self) -> PidHandle {
        if let Some(pid) = self.recycled.pop() {
            PidHandle(pid)
        } else {
            let handle = PidHandle(self.current);
            self.current += 1;
            handle
        }
    }
    fn dealloc(&mut self, pid: usize) {
        assert!(
            pid < self.current,
            "Pid to be deallocated has not been allocated!"
        );
        assert!(
            !(self.recycled.contains(&pid)),
            "Pid to be deallocated has been deallocated!"
        );
        self.recycled.push(pid);
    }
}

lazy_static! {
    static ref PID_ALLOCATOR: SyncRefCell<PidAllocator> = unsafe {
        SyncRefCell::new(PidAllocator {
            current: 0,
            recycled: Vec::new(),
        })
    };
}
impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_ALLOCATOR.borrow_mut().dealloc(self.0);
    }
}
pub struct KernelStack {
    pid: usize,
}
impl KernelStack {
    pub fn new(pid: usize) -> Self {
        let (bottom, top) = kernel_stack_range(pid);
        KERNEL_SPACE.borrow_mut().map_area(
            VirtAddr(bottom),
            VirtAddr(top),
            MapPermission::R | MapPermission::W,
            MapType::Framed,
        );
        Self { pid }
    }
    pub fn sp(&self) -> usize {
        let (_, sp) = kernel_stack_range(self.pid);
        sp
    }
    fn bottom(&self) -> usize {
        let (bottom, _) = kernel_stack_range(self.pid);
        bottom
    }
}
impl Drop for KernelStack {
    fn drop(&mut self) {
        KERNEL_SPACE
            .borrow_mut()
            .remove_area(VirtAddr(self.bottom()).floor());
    }
}
pub fn pid_alloc() -> PidHandle {
    PID_ALLOCATOR.borrow_mut().alloc()
}

pub fn kernel_stack_range(pid: usize) -> (usize, usize) {
    let top = TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}
