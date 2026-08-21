use crate::{config::KERNEL_STACK_SIZE, config::MAX_NUM_APP, link_app};

#[derive(Clone, Copy)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

static mut KERNEL_STACK: [KernelStack; MAX_NUM_APP] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_NUM_APP];

pub fn num_apps() -> usize {
    link_app::NUM_APPS
}

/// 返回第 id 个 app 的内核栈栈顶（trap 时切到该栈执行 handler）
pub fn kernel_sp(id: usize) -> usize {
    unsafe { KERNEL_STACK[id].data.as_ptr() as usize + KERNEL_STACK_SIZE }
}
