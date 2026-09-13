use crate::{config::STD_OUTPUT, mm::PageTable, sbi::putchar, task::current_user_token};

pub fn sys_write(fd: usize, buffer: *const u8, len: usize) -> isize {
    match fd {
        STD_OUTPUT => {
            let page_table = PageTable::from_token(current_user_token());
            let Some(buffers) = page_table.translated_byte_buffer(buffer, len) else {
                return -1;
            };
            for buf in buffers {
                for &byte in buf.iter() {
                    putchar(byte);
                }
            }
            len as isize
        }
        _ => -1,
    }
}
