use crate::config::TRAMPOLINE;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn init() -> Self {
        TaskContext {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }
    pub fn goto_restore(sp: usize) -> Self {
        unsafe extern "C" {
            fn __restore(context_address: usize) -> !;
            static strampoline: usize;
        }
        Self {
            ra: __restore as *const () as usize - unsafe { &strampoline as *const usize as usize }
                + TRAMPOLINE,
            sp,
            s: [0; 12],
        }
    }
}
