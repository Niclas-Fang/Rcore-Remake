use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{config::MEMORY_END, mm::address::{PhyAddr, PhyPageNum}, sync_refcell::SyncRefCell};

pub struct FrameTracker {
    pub ppn: PhyPageNum,
}

impl FrameTracker {
    fn new(ppn: PhyPageNum) -> Self {
        let data = ppn.get_bytes_array();
        for i in data {
            *i = 0
        }
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

pub struct StackFrameAllocator {
    current: usize,
    top: usize,
    recycled: Vec<usize>,
}
impl StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            top: 0,
            recycled: Vec::new(),
        }
    }
    fn init(&mut self, start: PhyPageNum, end: PhyPageNum) {
        self.current = start.0;
        self.top = end.0;
    }
    fn alloc(&mut self) -> Option<PhyPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(PhyPageNum(ppn))
        } else {
            if self.current == self.top {
                None
            } else {
                self.current += 1;
                Some(PhyPageNum(self.current - 1))
            }
        }
    }
    fn dealloc(&mut self, ppn: PhyPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: SyncRefCell<StackFrameAllocator> =
        unsafe { SyncRefCell::new(StackFrameAllocator::new()) };
}

pub fn init_frame_allocator() {
    unsafe extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.borrow_mut().init(
        PhyAddr(ekernel as *const () as usize ).ceil(),
        PhyAddr(MEMORY_END).floor(),
    );
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.borrow_mut().alloc().map(FrameTracker::new)
}
pub fn frame_dealloc(ppn: PhyPageNum) {
    FRAME_ALLOCATOR.borrow_mut().dealloc(ppn)
}
