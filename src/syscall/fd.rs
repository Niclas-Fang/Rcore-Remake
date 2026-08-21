use crate::{
    config::STD_OUTPUT,
    mm::PageTable,
    sbi::putchar,
    task::current_user_token,
};

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> isize {
    match fd {
        STD_OUTPUT => {
            // 用户缓冲区在用户页表里，内核页表没有映射，
            // 必须通过当前任务的用户页表逐页翻译成物理地址再读。
            let page_table = PageTable::from_token(current_user_token());
            let Some(buffers) = page_table.translated_byte_buffer(buffer, len) else {
                return -1; // 用户地址非法（近似 EFAULT）
            };
            for buf in buffers {
                for &byte in buf.iter() {
                    putchar(byte); // 字节流原样输出，不做 UTF-8 校验
                }
            }
            len as isize
        }
        _ => -1, // 不支持的 fd
    }
}
