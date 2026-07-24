//use core::arch::asm;
mod fd;
mod process;
//use crate::println;
const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;
const SYS_YIELD: usize = 124;

pub fn sys_call(which: usize, args: [usize; 3]) -> usize {
    //println!("call from user of {} with args {:?}", which, args);
    match which {
        SYS_WRITE => fd::sys_write(args[0], args[1] as *const u8, args[2]) as usize,
        SYS_EXIT => {
            process::sys_exit(args[0] as i32);
        }
        SYS_YIELD => {
            process::sys_yield()
        }
        _ => {
            panic!("Unsupported sys_call");
        }
    }
}
