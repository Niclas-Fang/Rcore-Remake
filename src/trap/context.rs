use riscv::register::sstatus::{self, SPP, Sstatus};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

impl TrapContext {
    fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    pub fn init(entry: usize, sp: usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut context = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
        };
        context.set_sp(sp);
        context
    }
}
