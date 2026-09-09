use alloc::{vec, vec::Vec};
use core::slice::from_raw_parts_mut;

use bitflags::bitflags;

use crate::mm::{
    address::{PAGE_SIZE, PhyAddr, PhyPageNum, VirtAddr, VirtPageNum},
    frame_allocator::{FrameTracker, frame_alloc},
};

bitflags! {
    #[derive(PartialEq,Clone, Copy)]
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Clone, Copy)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn new(ppn: PhyPageNum, flags: PTEFlags) -> Self {
        Self(ppn.0 << 10 | flags.bits() as usize)
    }
    pub fn empty() -> Self {
        Self(0)
    }
    pub fn ppn(&self) -> PhyPageNum {
        PhyPageNum(self.0 >> 10)
    }
    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.0 as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
    pub fn target(&self) -> Option<PhyPageNum> {
        if !self.is_valid() {
            return None;
        }
        Some(self.ppn())
    }
}
pub struct PageTable {
    root_ppn: PhyPageNum,
    frames: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = frame_alloc().expect("Unable to allocate root page table frame!");
        let root_ppn = frame.ppn;
        let page_table = Self {
            root_ppn,
            frames: vec![frame],
        };
        let root_phyaddr = Into::<PhyAddr>::into(root_ppn).0 as *mut PageTableEntry;
        unsafe {
            let ptes: &mut [PageTableEntry] = from_raw_parts_mut(root_phyaddr, PAGE_SIZE / 8);
            for pte in ptes {
                *pte = PageTableEntry::empty();
            }
        }
        page_table
    }
    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.index();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhyPageNum, flags: PTEFlags) {
        let [idx2, idx1, idx0] = vpn.index();
        let ptes = self.root_ppn.get_pte_array();
        let pte_l2 = &mut ptes[idx2];
        if !pte_l2.is_valid() {
            let frame = frame_alloc().expect("Unable to allocate new frame!");
            *pte_l2 = PageTableEntry::new(frame.ppn, PTEFlags::V);
            self.frames.push(frame);
        }

        let ptes = pte_l2.ppn();
        let ptes = ptes.get_pte_array();
        let pte_l1 = &mut ptes[idx1];
        if !pte_l1.is_valid() {
            let frame = frame_alloc().expect("Unable to allocate new frame!");
            *pte_l1 = PageTableEntry::new(frame.ppn, PTEFlags::V);
            self.frames.push(frame);
        }

        let ptes = pte_l1.ppn();
        let ptes = ptes.get_pte_array();
        if ptes[idx0].is_valid() {
            panic!("This virtual page has been mapped to an existent physical page!")
        }
        ptes[idx0] = PageTableEntry::new(ppn, flags | PTEFlags::V);
    }
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        if let Some(pte) = self.find_pte(vpn) {
            *pte = PageTableEntry::empty()
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PhyPageNum> {
        let pte = self.find_pte(vpn)?;
        if !pte.is_valid() {
            return None;
        }
        Some(pte.ppn())
    }
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhyPageNum(satp & ((1 << 44) - 1)),
            frames: Vec::new(),
        }
    }
    pub fn token(&self) -> usize {
        (8usize << 60) | self.root_ppn.0
    }
    /// 把用户虚拟地址 `ptr` 起 `len` 字节的缓冲区，按页切成物理上
    /// 连续的切片（内核不映射用户空间，访问用户缓冲必须经此翻译）。
    /// 任一页翻译失败（非法地址）返回 `None`。
    pub fn translated_byte_buffer(
        &self,
        ptr: *const u8,
        len: usize,
    ) -> Option<Vec<&'static mut [u8]>> {
        let mut start = ptr as usize;
        let end = start.checked_add(len)?; // 溢出检查
        let mut v = Vec::new();
        while start < end {
            let start_va = VirtAddr(start);
            let ppn = self.translate(start_va.floor())?;
            let end_va = VirtAddr(end.min((start + PAGE_SIZE) - (start & (PAGE_SIZE - 1))));
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
            start = end_va.0;
        }
        Some(v)
    }
}
