use core::range::Range;
use lazy_static::lazy_static;
use crate::config::MEMORY_END;

use alloc::vec;
use alloc::vec::Vec;
use riscv::{
    asm::sfence_vma_all,
    register::satp::{self, Satp},
};

use crate::{
    config::TRAMPOLINE,
    mm::{
        address::{PhyPageNum, VirtAddr, VirtPageNum},
        frame_allocator::{FrameTracker, frame_alloc},
        page_table::{PTEFlags, PageTable},
    },
};

pub enum MapType {
    Identical,
    Framed,
}

pub type MapPermission = PTEFlags;

pub struct MapArea {
    vpn_range: Range<VirtPageNum>,
    data_frames: Vec<FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

impl MapArea {
    pub fn new(map_type: MapType, map_perm: MapPermission, start: VirtAddr, end: VirtAddr) -> Self {
        Self {
            vpn_range: Range {
                start: start.floor(),
                end: end.ceil(),
            },
            data_frames: vec![],
            map_type,
            map_perm,
        }
    }
    pub fn map_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        let ppn = match self.map_type {
            MapType::Identical => PhyPageNum(vpn.0),
            MapType::Framed => {
                let frame_tracker = frame_alloc().unwrap();
                let ppn = frame_tracker.ppn;
                self.data_frames.push(frame_tracker);
                ppn
            }
        };
        page_table.map(vpn, ppn, self.map_perm);
    }
    pub fn unmap_one(&mut self, page_table: &mut PageTable, vpn: VirtPageNum) {
        page_table.unmap(vpn);
    }
    pub fn map(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range.start.0..self.vpn_range.end.0 {
            self.map_one(page_table, VirtPageNum(vpn));
        }
    }
    pub fn unmap(&mut self, page_table: &mut PageTable) {
        for vpn in self.vpn_range.start.0..self.vpn_range.end.0 {
            self.unmap_one(page_table, VirtPageNum(vpn));
        }
    }
}

pub struct MemorySet {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

impl MemorySet {
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            areas: vec![],
        }
    }
    pub fn map_area(
        &mut self,
        start: VirtAddr,
        end: VirtAddr,
        perm: MapPermission,
        type_: MapType,
    ) {
        let mut area = MapArea::new(type_, perm, start, end);
        area.map(&mut self.page_table);
        self.areas.push(area);
    }
    pub fn token(&self) -> usize {
        self.page_table.token()
    }
    pub fn activate(&self) {
        unsafe { satp::write(Satp::from_bits(self.token())) };
        sfence_vma_all();
    }
    pub fn map_trampoline(&mut self) {
        unsafe extern "C" {
            static strampoline: u8;
        }
        let trampoline = unsafe { &strampoline as *const u8 as usize };
        self.page_table.map(
            VirtPageNum(TRAMPOLINE),
            PhyPageNum(trampoline),
            PTEFlags::R | PTEFlags::U | PTEFlags::X,
        );
    }
    pub fn new_kernel() -> Self {
        unsafe extern "C" {
            static stext: usize;
            static etext: usize;
            static srodata: usize;
            static erodata: usize;
            static sdata: usize;
            static edata: usize;
            static sbss: usize;
            static ebss: usize;
            static ekernel: usize;
        }
        let mut memory_set = Self::new();
        unsafe {
            memory_set.map_area(
                VirtAddr(&stext as *const usize as usize),
                VirtAddr(&etext as *const usize as usize),
                MapPermission::R | MapPermission::X,
                MapType::Identical,
            );
            memory_set.map_area(
                VirtAddr(&srodata as *const usize as usize),
                VirtAddr(&erodata as *const usize as usize),
                MapPermission::R,
                MapType::Identical,
            );
            memory_set.map_area(
                VirtAddr(&sdata as *const usize as usize),
                VirtAddr(&edata as *const usize as usize),
                MapPermission::R | MapPermission::W,
                MapType::Identical,
            );
            memory_set.map_area(
                VirtAddr(&sbss as *const usize as usize),
                VirtAddr(&ebss as *const usize as usize),
                MapPermission::R | MapPermission::W,
                MapType::Identical,
            );
            memory_set.map_area(
                VirtAddr(&ekernel as *const usize as usize),
                VirtAddr(MEMORY_END),
                MapPermission::R | MapPermission::W,
                MapType::Identical,
            );
            memory_set.map_trampoline();
            memory_set
        }
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: MemorySet = MemorySet::new_kernel();
}
