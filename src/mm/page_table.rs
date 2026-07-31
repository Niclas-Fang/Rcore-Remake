use core::slice::from_raw_parts_mut;

use bitflags::bitflags;

use crate::mm::address::{PAGE_SIZE, PhyAddr, PhyPageNum, VirtPageNum};

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
        Self {
            0: ppn.0 << 10 | flags.bits() as usize,
        }
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
}

impl PageTable {
    pub fn new(root_ppn: PhyPageNum) -> Self {
        let root_phyaddr = Into::<PhyAddr>::into(root_ppn).0 as *mut PageTableEntry;
        unsafe {
            let ptes: &mut [PageTableEntry] = from_raw_parts_mut(root_phyaddr, PAGE_SIZE / 8);
            for pte in ptes {
                *pte = PageTableEntry::empty();
            }
        }
        Self { root_ppn }
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
            *pte_l2 = alloc_page();
        }

        let ptes = pte_l2.ppn();
        let ptes = ptes.get_pte_array();
        let pte_l1 = &mut ptes[idx1];
        if !pte_l1.is_valid() {
            *pte_l1 = alloc_page();
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
        if !pte.is_valid() {return None;}
        Some(pte.ppn())
    }
    pub fn from_token(satp: usize) -> Self {
        Self {
            root_ppn: PhyPageNum(satp & ((1 << 44) - 1)),
        }
    }
}

fn alloc_page() -> PageTableEntry {
    unimplemented!()
}
