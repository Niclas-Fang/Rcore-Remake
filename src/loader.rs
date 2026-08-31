use crate::{
    config::{KERNEL_STACK_SIZE, MAX_NUM_APP},
    link_app, println,
};

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

pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    let index = link_app::APP_NAMES.iter().position(|&x| x == name)?;
    Some(link_app::APPS[index])
}

pub fn list_apps() {
    println!("{:?}", link_app::APP_NAMES);
}
