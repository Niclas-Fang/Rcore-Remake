use core::slice::from_raw_parts_mut;

use crate::mm::page_table::PageTableEntry;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhyAddr(pub usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtAddr(pub usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhyPageNum(pub usize);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VirtPageNum(pub usize);

pub const PAGE_SIZE: usize = 1 << 12;
pub const PAGE_SIZE_BIT: usize = 12;

impl PhyAddr {
    fn offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}
impl VirtAddr {
    fn offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
}
impl PhyPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let phy_addr: PhyAddr = (*self).into();
        unsafe { from_raw_parts_mut(phy_addr.0 as *mut PageTableEntry, PAGE_SIZE / 8) }
    }
}
impl VirtPageNum {
    pub fn index(&self) -> [usize; 3] {
        let vpn = self.0;
        let idx0 = (vpn >> 0) & 0x1ff;
        let idx1 = (vpn >> 9) & 0x1ff;
        let idx2 = (vpn >> 18) & 0x1ff;
        [idx2, idx1, idx0]
    }
}
impl From<PhyAddr> for PhyPageNum {
    fn from(value: PhyAddr) -> Self {
        assert!(value.offset() == 0);
        Self(value.0 >> 12)
    }
}

impl From<PhyPageNum> for PhyAddr {
    fn from(value: PhyPageNum) -> Self {
        Self(value.0 << 12)
    }
}

impl From<VirtAddr> for VirtPageNum {
    fn from(value: VirtAddr) -> Self {
        assert!(value.offset() == 0);
        Self(value.0 >> 12)
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(value: VirtPageNum) -> Self {
        Self(value.0 << 12)
    }
}
