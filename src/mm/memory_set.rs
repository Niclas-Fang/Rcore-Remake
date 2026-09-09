use crate::{
    config::{MEMORY_END, PAGE_SIZE, TRAP_CONTEXT, USER_BASE_VA, USER_STACK_SIZE},
    mm::address::PhyAddr,
    sync_refcell::SyncRefCell,
};
use core::ptr::copy;
use core::range::Range;
use lazy_static::lazy_static;

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
#[derive(Clone, Copy)]
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
        map_perm: MapPermission,
        map_type: MapType,
    ) {
        let mut area = MapArea::new(map_type, map_perm, start, end);
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
            VirtAddr(TRAMPOLINE).floor(),
            PhyAddr(trampoline).ceil(),
            PTEFlags::R | PTEFlags::X,
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
        let mut kernel = Self::new();
        unsafe {
            kernel.map_area(
                VirtAddr(&stext as *const usize as usize),
                VirtAddr(&etext as *const usize as usize),
                MapPermission::R | MapPermission::X,
                MapType::Identical,
            );
            kernel.map_area(
                VirtAddr(&srodata as *const usize as usize),
                VirtAddr(&erodata as *const usize as usize),
                MapPermission::R,
                MapType::Identical,
            );
            kernel.map_area(
                VirtAddr(&sdata as *const usize as usize),
                VirtAddr(&edata as *const usize as usize),
                MapPermission::R | MapPermission::W,
                MapType::Identical,
            );
            kernel.map_area(
                VirtAddr(&sbss as *const usize as usize),
                VirtAddr(&ebss as *const usize as usize),
                MapPermission::R | MapPermission::W,
                MapType::Identical,
            );
            kernel.map_area(
                VirtAddr(&ekernel as *const usize as usize),
                VirtAddr(MEMORY_END),
                MapPermission::R | MapPermission::W,
                MapType::Identical,
            );
            kernel.map_trampoline();
            kernel
        }
    }
    pub fn from_bin(bin: &[u8]) -> Self {
        let mut app = MemorySet::new();
        let len = bin.len();
        app.map_trampoline();
        app.map_area(
            VirtAddr(USER_BASE_VA),
            VirtAddr(USER_BASE_VA + len),
            MapPermission::R | MapPermission::W | MapPermission::X | MapPermission::U,
            MapType::Framed,
        );
        app.map_area(
            VirtAddr(TRAP_CONTEXT - USER_STACK_SIZE),
            VirtAddr(TRAP_CONTEXT),
            MapPermission::W | MapPermission::R | MapPermission::U,
            MapType::Framed,
        );
        app.map_area(
            VirtAddr(TRAP_CONTEXT),
            VirtAddr(TRAMPOLINE),
            MapPermission::R | MapPermission::W,
            MapType::Framed,
        );
        app.copy_bin(bin);
        app
    }
    fn copy_bin(&mut self, bin: &[u8]) {
        let vpn_base = VirtAddr(USER_BASE_VA).floor();
        for (i, chunk) in bin.chunks(PAGE_SIZE).enumerate() {
            let vpn = VirtPageNum(vpn_base.0 + i);
            let ppn = self
                .page_table
                .translate(vpn)
                .expect("App text stack overflow!");
            unsafe { copy(chunk.as_ptr(), (ppn.0 << 12) as *mut u8, chunk.len()) }
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PhyPageNum> {
        self.page_table.translate(vpn)
    }
    pub fn remap_trap_cx(&mut self, trap_cx_ppn: PhyPageNum) {
        self.page_table.unmap(VirtAddr(TRAP_CONTEXT).floor());
        self.page_table.map(
            VirtAddr(TRAP_CONTEXT).floor(),
            trap_cx_ppn,
            PTEFlags::R | PTEFlags::W,
        );
        sfence_vma_all();
    }
    pub fn remove_area(&mut self, start_vpn: VirtPageNum) {
        let index = self
            .areas
            .iter()
            .position(|x| x.vpn_range.start == start_vpn)
            .expect("Cannot find map area with given vpn {pid}!");
        self.areas[index].unmap(&mut self.page_table);
        self.areas.remove(index);
    }
    pub fn from_parent(parent: &Self) -> Self {
        let mut child = Self::new();
        child.map_trampoline();
        for area in &parent.areas {
            let start = VirtAddr(area.vpn_range.start.0 << 12);
            let end = VirtAddr(area.vpn_range.end.0 << 12);
            let map_perm = area.map_perm;
            let map_type = area.map_type;
            child.map_area(start, end, map_perm, map_type);
            for vpn in area.vpn_range.start.0..area.vpn_range.end.0 {
                let parent_ppn = parent.page_table.translate(VirtPageNum(vpn)).unwrap();
                let child_ppn = child.page_table.translate(VirtPageNum(vpn)).unwrap();
                unsafe {
                    copy(
                        (parent_ppn.0 << 12) as *const u8,
                        (child_ppn.0 << 12) as *mut u8,
                        PAGE_SIZE,
                    );
                }
            }
        }
        child
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: SyncRefCell<MemorySet> = {
        let kernel = MemorySet::new_kernel();
        unsafe { SyncRefCell::new(kernel) }
    };
}

pub fn kernel_satp() -> usize {
    KERNEL_SPACE.borrow().token()
}
pub fn remap_trap_context(trap_cx_ppn: PhyPageNum) {
    KERNEL_SPACE.borrow_mut().remap_trap_cx(trap_cx_ppn);
}
pub fn map_kernel_area(start: VirtAddr, end: VirtAddr, perm: MapPermission, type_: MapType) {
    KERNEL_SPACE.borrow_mut().map_area(start, end, perm, type_);
}
pub fn activate() {
    KERNEL_SPACE.borrow().activate();
}
pub fn remove_area(start_vpn: VirtPageNum) {
    KERNEL_SPACE.borrow_mut().remove_area(start_vpn);
}
