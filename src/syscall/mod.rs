mod fd;
mod process;

use crate::config::{SYS_EXIT, SYS_GET_TIME, SYS_WRITE, SYS_YIELD};

pub fn sys_call(which: usize, args: [usize; 3]) -> usize {
    match which {
        SYS_WRITE => fd::sys_write(args[0], args[1] as *const u8, args[2]) as usize,
        SYS_EXIT => {
            process::sys_exit(args[0] as i32);
        }
        SYS_YIELD => process::sys_yield(),
        SYS_GET_TIME => process::sys_get_time(),
        _ => {
            panic!("Unsupported sys_call");
        }
    }
}
