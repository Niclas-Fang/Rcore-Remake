use core::slice::from_raw_parts;

use crate::{
    config::{PAGE_SIZE, STD_OUTPUT},
    mm::{PageTable, VirtAddr},
    task::current_user_token,
};

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> isize {
    match fd {
        STD_OUTPUT => {
            // 用户缓冲区在用户页表里，内核页表没有映射，
            // 直接用用户虚拟地址会触发内核态 load fault。
            // 必须通过当前任务的用户页表逐页翻译成物理地址再读。
            let page_table = PageTable::from_token(current_user_token());
            let mut remaining = len;
            let mut ptr = buffer as usize;
            while remaining > 0 {
                let vpn = VirtAddr(ptr).floor();
                let Some(ppn) = page_table.translate(vpn) else {
                    break;
                };
                let offset = ptr & (PAGE_SIZE - 1);
                let chunk = (PAGE_SIZE - offset).min(remaining);
                let src = unsafe { from_raw_parts((ppn.0 << 12 | offset) as *const u8, chunk) };
                let str = core::str::from_utf8(src).unwrap();
                crate::print!("{str}");
                remaining -= chunk;
                ptr += chunk;
            }
            len as isize
        }
        _ => {
            panic!("Unsupported output file descripter!");
        }
    }
}
