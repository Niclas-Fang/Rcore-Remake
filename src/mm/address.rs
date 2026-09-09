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

pub use crate::config::PAGE_SIZE;

impl PhyAddr {
    fn offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    /// Get the (floor) physical page number
    pub fn floor(&self) -> PhyPageNum {
        PhyPageNum(self.0 / PAGE_SIZE)
    }
    /// Get the (ceil) physical page number
    pub fn ceil(&self) -> PhyPageNum {
        PhyPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }
}
impl VirtAddr {
    pub fn page_offset(&self) -> usize {
        self.0 & (PAGE_SIZE - 1)
    }
    /// Get the (floor) virtual page number
    pub fn floor(&self) -> VirtPageNum {
        VirtPageNum(self.0 / PAGE_SIZE)
    }
    /// Get the (ceil) virtual page number
    pub fn ceil(&self) -> VirtPageNum {
        VirtPageNum((self.0 - 1 + PAGE_SIZE) / PAGE_SIZE)
    }
}
impl PhyPageNum {
    pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
        let phy_addr: PhyAddr = (*self).into();
        unsafe { from_raw_parts_mut(phy_addr.0 as *mut PageTableEntry, PAGE_SIZE / 8) }
    }
    pub fn get_bytes_array(&self) -> &'static mut [u8] {
        let pa: PhyAddr = (*self).into();
        unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, PAGE_SIZE) }
    }
}
impl VirtPageNum {
    pub fn index(&self) -> [usize; 3] {
        let vpn = self.0;
        let idx0 = vpn & 0x1ff;
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
        assert!(value.page_offset() == 0);
        Self(value.0 >> 12)
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(value: VirtPageNum) -> Self {
        Self(value.0 << 12)
    }
}
