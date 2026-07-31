#[derive(Debug,Clone,Copy,PartialEq)]
pub struct PhyAddr(pub usize);
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct VirtAddr(pub usize);
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct PhyPageNum(pub usize);
#[derive(Debug,Clone,Copy,PartialEq)]
pub struct VirtPageNum(pub usize);

const PAGE_SIZE: usize = 1 << 12;
const PAGE_SIZE_BIT: usize = 12;

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
        assert!(value.offset() != 0);
        Self(value.0 >> 12)
    }
}

impl From<VirtPageNum> for VirtAddr {
    fn from(value: VirtPageNum) -> Self {
        Self(value.0 << 12)
    }
}
