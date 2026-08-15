use riscv::register::sstatus::{self, SPP, Sstatus};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    pub kernel_satp: usize,
    pub user_satp: usize,
    pub kernel_sp: usize,
    pub trap_handler: usize,
}

impl TrapContext {
    fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    pub fn init(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        user_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        let mut context = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_satp: kernel_satp,
            user_satp: user_satp,
            kernel_sp: kernel_sp,
            trap_handler: trap_handler,
        };
        context.set_sp(sp);
        context
    }
}
