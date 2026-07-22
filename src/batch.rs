use core::ptr::copy;

use crate::link_app::{APPS, NUM_APPS};
use crate::trap::{TrapContext, goto_user};
use crate::{println, sbi};

static mut CURRENT_APP: usize = 0;
static USER_BASE: usize = 0x80400000;
static USER_STACK_TOP: usize = 0x84400000;

pub fn run_next_app() -> ! {
    if unsafe { CURRENT_APP } >= NUM_APPS {
        println!("[kernel] All apps already run");
        sbi::shutdown();
    }
    println!("[kernel] Running app {}", unsafe { CURRENT_APP });
    let app = APPS[unsafe { CURRENT_APP }];
    unsafe { copy(app.as_ptr(), USER_BASE as *mut u8, app.len()) };
    unsafe {
        CURRENT_APP += 1;
    }
    let context = TrapContext::init(USER_BASE, USER_STACK_TOP);
    goto_user(context);
}
