use crate::{
    config::{MEMORY_END, PAGE_SIZE, TRAP_CONTEXT, USER_BASE_VA, USER_STACK_SIZE},
    link_app,
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
    pub fn from_app(app_id: usize) -> Self {
        let mut memory_set = MemorySet::new();
        let len = link_app::APPS[app_id].len();
        memory_set.map_trampoline();
        memory_set.map_area(
            VirtAddr(USER_BASE_VA),
            VirtAddr(USER_BASE_VA + len),
            MapPermission::R | MapPermission::W | MapPermission::X | MapPermission::U,
            MapType::Framed,
        );
        memory_set.map_area(
            VirtAddr(TRAP_CONTEXT - USER_STACK_SIZE),
            VirtAddr(TRAP_CONTEXT),
            MapPermission::W | MapPermission::R | MapPermission::U,
            MapType::Framed,
        );
        memory_set.map_area(
            VirtAddr(TRAP_CONTEXT),
            VirtAddr(TRAMPOLINE),
            MapPermission::R | MapPermission::W,
            MapType::Framed,
        );
        memory_set.copy_data(app_id);
        memory_set
    }
    fn copy_data(&mut self, app_id: usize) {
        if app_id >= link_app::NUM_APPS {
            panic!("APP number exceeds existing numbers!")
        }
        let app = link_app::APPS[app_id];
        let va_base = VirtAddr(USER_BASE_VA).floor();
        for (i, chunk) in app.chunks(PAGE_SIZE).enumerate() {
            let va = VirtPageNum(va_base.0 + i);
            let pa = self
                .page_table
                .translate(va)
                .expect("App text stack overflow!");
            unsafe { copy(chunk.as_ptr(), pa.0 as *mut u8, chunk.len()) }
        }
    }
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PhyPageNum> {
        self.page_table.translate(vpn)
    }
    pub fn remap_trap_context(&mut self, trap_cx_ppn: PhyPageNum) {
        self.page_table.unmap(VirtPageNum(TRAP_CONTEXT));
        self.page_table.map(
            VirtPageNum(TRAP_CONTEXT),
            trap_cx_ppn,
            PTEFlags::R | PTEFlags::W,
        );
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
