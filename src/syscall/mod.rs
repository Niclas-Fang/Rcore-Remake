//use core::arch::asm;

use crate::println;
const SYS_WRITE: usize = 64;
const SYS_EXIT: usize = 93;

pub fn sys_call(which: usize, args: [usize; 3]) -> usize {
    println!("call from user of {} with args {:?}",which, args);
    unimplemented!()
}

