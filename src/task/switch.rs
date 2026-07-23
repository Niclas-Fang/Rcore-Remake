use core::arch::global_asm;

use super::context::TaskContext;


global_asm!(include_str!("switch.S"));
unsafe extern "C" {
    fn __switch(src: *mut TaskContext, des: *mut TaskContext);
}