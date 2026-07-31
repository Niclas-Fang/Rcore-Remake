use core::ptr::copy;

use crate::{
    config::{
        APP_SIZE, KERNEL_STACK_SIZE, MAX_NUM_APP, USER_BASE, USER_STACK_SIZE,
    },
    link_app, trap::TrapContext,
};

#[derive(Clone, Copy)]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}
#[derive(Clone, Copy)]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

static KERNEL_STACK: [KernelStack; MAX_NUM_APP] = [KernelStack {
    data: [0; KERNEL_STACK_SIZE],
}; MAX_NUM_APP];

static USER_STACK: [UserStack; MAX_NUM_APP] = [UserStack {
    data: [0; USER_STACK_SIZE],
}; MAX_NUM_APP];

pub fn num_apps() -> usize {
    link_app::NUM_APPS
}

impl KernelStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    fn push_context(&self, context: TrapContext) -> usize {
        let addr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *addr = context;
        }
        addr as usize
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub fn load_apps() {
    let apps = link_app::APPS;
    for (id, app) in apps.iter().enumerate() {
        unsafe {
            copy(app.as_ptr(), app_base(id) as *mut u8, app.len());
        }
    }
}

fn app_base(id: usize) -> usize {
    USER_BASE + id * APP_SIZE
}

pub fn init_app_cx(app_id: usize) -> usize {
    KERNEL_STACK[app_id].push_context(TrapContext::init(
        app_base(app_id),
        USER_STACK[app_id].get_sp(),
    ))
}
